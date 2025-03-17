use std::collections::HashMap;

use super::node::{
    Add, AddSub, Assign, Compare, Equality, Equals, Expr, For, Ident, If, Lvar, Mul, MulDiv,
    MultiStmt, Primary, PrimaryNode, Program, Relational, Statement, Stmt, Unary, While,
};
const IDENTITY_OFFSET: usize = 8;
const RETURN: &str = "return";
const IF: &str = "if";
const WHILE: &str = "while";
const FOR: &str = "for";

#[derive(Debug)]
pub struct ParseFailure {
    pub index: usize,
    pub read_line: usize,
    pub source: Option<String>,
    pub reason: String,
}
pub type ParseResult<T> = Result<T, ParseFailure>;

pub struct Parser<'a> {
    pub index: usize,
    pub input: &'a String,
    pub ident_count: usize,
    pub idents: HashMap<String, usize>,
    pub read_lines: usize,
    pub line_index: usize,
}
trait IsToken {
    fn is_token_parts(self) -> bool;
}
impl IsToken for char {
    fn is_token_parts(self) -> bool {
        self.is_ascii_alphanumeric() || self == '_'
    }
}
impl Parser<'_> {
    fn succ(&mut self, count: usize) {
        self.index += count;
        self.line_index += count;
    }
    fn consume(&mut self, str: &str) -> Option<String> {
        self.space();
        let end = self.index + str.len();
        let taken = self.input.get(self.index..end)?;
        if taken.eq(str) {
            self.succ(str.len());
            Some(taken.into())
        } else {
            None
        }
    }
    fn consume_f(&mut self, checker: fn(check: char) -> bool) -> Option<String> {
        self.space();
        let mut str = "".to_string();
        loop {
            if self.empty() {
                break;
            }
            match self.top_f(checker) {
                None => break,
                Some(c) => {
                    str.push(c);
                }
            };
        }
        if str.is_empty() { None } else { Some(str) }
    }
    fn consume_expect(&mut self, checker: fn(check: char) -> bool, expect: &str) -> Option<String> {
        self.space();
        let mut str = "".to_string();
        let mut offset = 0;
        loop {
            if self.empty() {
                break;
            }
            let c = self.input.chars().nth(self.index + offset);
            if c.is_none() || !checker(c.unwrap()) {
                break;
            }
            offset += 1;
            str.push(c.unwrap())
        }
        if !str.eq(expect) {
            return None;
        }
        self.succ(str.len());
        Some(str)
    }

    fn empty(&mut self) -> bool {
        self.space();
        self.index == self.input.len()
    }
    fn fail(&self, reason: String) -> ParseFailure {
        let line = match self.input.split('\n').nth(self.read_lines) {
            Some(str) => Some(str.into()),
            _ => None,
        };
        ParseFailure {
            index: self.line_index,
            read_line: self.read_lines,
            source: line,
            reason: reason,
        }
    }
    fn check_top(&mut self, var: &str) -> bool {
        self.space();
        self.input
            .chars()
            .skip(self.index)
            .take(var.len())
            .eq(var.chars())
    }
    fn check_top_f(&mut self, checker: fn(check: char) -> bool) -> bool {
        self.space();
        match self.input.chars().nth(self.index) {
            None => false,
            Some(c) => checker(c),
        }
    }
    fn top_f(&mut self, checker: fn(check: char) -> bool) -> Option<char> {
        self.space();
        match self.input.chars().nth(self.index) {
            None => None,
            Some(c) => {
                if checker(c) {
                    self.succ(1);
                    Some(c)
                } else {
                    None
                }
            }
        }
    }

    fn space(&mut self) {
        loop {
            if self
                .input
                .chars()
                .skip(self.index)
                .take(2)
                .eq("\r\n".chars())
            {
                self.read_lines += 1;
                self.succ(2);
                self.line_index = 0;
                continue;
            }
            match self.input.chars().nth(self.index) {
                Some(' ') => {
                    self.succ(1);
                }
                Some('\r') | Some('\n') => {
                    self.read_lines += 1;
                    self.succ(1);
                    self.line_index = 0;
                }
                _ => break,
            }
        }
    }
    fn p_exp(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        match self.expr() {
            Err(e) => Err(e),
            Ok(e) => {
                if self.consume(")").is_none() {
                    return Err(self.fail("parenthesis unbalanced".into()));
                }
                Ok(Primary {
                    ope: ope,
                    node: PrimaryNode::Expr(Box::new(e)),
                })
            }
        }
    }
    fn p_num(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        self.space();
        let mut raw_num: String = "".into();
        loop {
            if self.empty() {
                break;
            }
            match self.top_f(|c| c.is_numeric()) {
                None => break,
                Some(c) => raw_num.push(c),
            }
        }
        if raw_num.len() == 0 {
            return Err(self.fail("number expected".into()));
        }
        Ok(Primary {
            ope: ope,
            node: PrimaryNode::Num(raw_num),
        })
    }
    fn p_ident(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        match self.consume_f(|c| c.is_token_parts()) {
            None => Err(self.fail(
                "identity expected, but identity can only contain 'a-z', 'A-Z', '0-9' and '_'"
                    .into(),
            )),

            Some(i) => {
                let offset = match self.idents.get(&i) {
                    None => {
                        self.ident_count += 1;
                        let o = self.ident_count * 8;
                        self.idents.insert(i, o);
                        o
                    }
                    Some(ofs) => *ofs,
                };

                Ok(Primary {
                    ope: ope,
                    node: PrimaryNode::Lv(Lvar::Id(Ident { offset: offset })),
                })
            }
        }
    }
    fn primary(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        if self.empty() {
            return Err(self.fail("number or ( expected".into()));
        }
        if self.consume("(").is_some() {
            return self.p_exp(ope);
        }
        if self.check_top_f(|c| c.is_numeric()) {
            self.p_num(ope)
        } else {
            self.p_ident(ope)
        }
    }
    fn unary(&mut self, ope: Option<MulDiv>) -> ParseResult<Unary> {
        if self.empty() {
            return Err(self.fail("+, -, num or expression expected".into()));
        }
        let addsub = if self.consume("+").is_some() {
            Some(AddSub::Plus)
        } else if self.consume("-").is_some() {
            Some(AddSub::Minus)
        } else {
            None
        };
        match self.primary(addsub) {
            Ok(p) => Ok(Unary { ope: ope, prim: p }),
            Err(e) => Err(e),
        }
    }
    fn mul(&mut self, ope: Option<AddSub>) -> ParseResult<Mul> {
        let first = self.unary(None);
        if first.is_err() {
            return Err(first.unwrap_err());
        }
        let mut eq = Mul {
            first: first.unwrap(),
            ope: ope,
            unarys: Vec::new(),
        };
        loop {
            if self.empty() {
                break;
            }
            let ope = if self.consume("*").is_some() {
                Some(MulDiv::Multi)
            } else if self.consume("/").is_some() {
                Some(MulDiv::Divide)
            } else {
                None
            };
            if ope.is_none() {
                break;
            }
            match self.unary(ope) {
                Err(e) => {
                    return Err(e);
                }
                Ok(res) => eq.unarys.push(res),
            }
        }
        Ok(eq)
    }
    fn add(&mut self, ope: Option<Compare>) -> ParseResult<Add> {
        let first = self.mul(None);
        if first.is_err() {
            return Err(first.unwrap_err());
        }
        let mut eq = Add {
            first: first.unwrap(),
            ope: ope,
            muls: Vec::new(),
        };
        loop {
            if self.empty() {
                break;
            }
            let ope = if self.consume("+").is_some() {
                Some(AddSub::Plus)
            } else if self.consume("-").is_some() {
                Some(AddSub::Minus)
            } else {
                None
            };
            if ope.is_none() {
                break;
            }
            match self.mul(ope) {
                Err(e) => {
                    return Err(e);
                }
                Ok(res) => eq.muls.push(res),
            }
        }
        Ok(eq)
    }
    fn relational(&mut self, ope: Option<Equals>) -> ParseResult<Relational> {
        let first = self.add(None);
        if first.is_err() {
            return Err(first.unwrap_err());
        }
        let mut eq = Relational {
            first: first.unwrap(),
            ope: ope,
            adds: Vec::new(),
        };
        loop {
            if self.empty() {
                break;
            }
            let ope = if self.consume(">=").is_some() {
                Some(Compare::Gte)
            } else if self.consume("<=").is_some() {
                Some(Compare::Lte)
            } else if self.consume("<").is_some() {
                Some(Compare::Lt)
            } else if self.consume(">").is_some() {
                Some(Compare::Gt)
            } else {
                None
            };
            if ope.is_none() {
                break;
            }
            match self.add(ope) {
                Err(e) => {
                    return Err(e);
                }
                Ok(res) => eq.adds.push(res),
            }
        }
        Ok(eq)
    }
    fn equality(&mut self) -> ParseResult<Equality> {
        let first = self.relational(None);
        if first.is_err() {
            return Err(first.unwrap_err());
        }
        let mut eq = Equality {
            first: first.unwrap(),
            relationals: Vec::new(),
        };
        loop {
            if self.empty() {
                break;
            }
            let ope = if self.consume("==").is_some() {
                Some(Equals::Equal)
            } else if self.consume("!=").is_some() {
                Some(Equals::NotEqual)
            } else {
                None
            };
            if ope.is_none() {
                break;
            }
            match self.relational(ope) {
                Err(e) => {
                    return Err(e);
                }
                Ok(res) => eq.relationals.push(res),
            }
        }
        Ok(eq)
    }
    fn assign(&mut self) -> ParseResult<Assign> {
        let eq = self.equality();
        if eq.is_err() {
            return Err(eq.unwrap_err());
        }
        if self.consume("=").is_none() {
            return Ok(Assign {
                lvar: eq.unwrap(),
                rvar: None,
            });
        }
        match self.assign() {
            Ok(a) => Ok(Assign {
                lvar: eq.unwrap(),
                rvar: Some(Box::new(a)),
            }),
            Err(e) => Err(e),
        }
    }
    fn expr(&mut self) -> ParseResult<Expr> {
        let ret = self.consume_expect(|c| c.is_token_parts(), RETURN);
        match self.assign() {
            Ok(a) => Ok(Expr {
                assign: a,
                ret: ret.is_some(),
            }),
            Err(e) => Err(e),
        }
    }
    fn while_(&mut self) -> ParseResult<While> {
        if self.consume("(").is_none() {
            return Err(self.fail("( expected before 'while'".into()));
        }
        let cond = self.expr()?;
        if self.consume(")").is_none() {
            return Err(self.fail(") expected after 'while'".into()));
        }
        let stmt = self.stmt()?;
        Ok(While {
            cond,
            stmt: Box::new(stmt),
        })
    }
    fn for_(&mut self) -> ParseResult<For> {
        if self.consume("(").is_none() {
            return Err(self.fail("( expected before 'for'".into()));
        }
        let init = if self.check_top(";") {
            None
        } else {
            Some(self.expr()?)
        };
        if self.consume(";").is_none() {
            return Err(self.fail("; expected after for initialize section".into()));
        }
        let cond = if self.check_top(";") {
            None
        } else {
            Some(self.expr()?)
        };
        if self.consume(";").is_none() {
            return Err(self.fail("; expected after for condition section".into()));
        }
        let step = if self.check_top(")") {
            None
        } else {
            Some(self.expr()?)
        };
        if self.consume(")").is_none() {
            return Err(self.fail(") expected after 'for'".into()));
        }
        Ok(For {
            init: init,
            cond: cond,
            step: step,
            stmt: Box::new(self.stmt()?),
        })
    }
    fn if_(&mut self) -> ParseResult<If> {
        if self.consume("(").is_none() {
            return Err(self.fail("( expected before 'if'".into()));
        }
        let cond = self.expr()?;
        if self.consume(")").is_none() {
            return Err(self.fail(") expected after 'if'".into()));
        }
        let stmt = self.stmt()?;
        if self.consume("else").is_none() {
            Ok(If {
                cond,
                stmt: Box::new(stmt),
                else_: None,
            })
        } else {
            Ok(If {
                cond,
                stmt: Box::new(stmt),
                else_: Some(Box::new(self.stmt()?)),
            })
        }
    }
    fn multi_stmts(&mut self) -> ParseResult<MultiStmt> {
        let mut stmts = Vec::new();
        loop {
            if self.check_top("}") || self.empty() {
                break;
            };
            stmts.push(self.stmt()?);
        }
        if self.consume("}").is_none() {
            return Err(self.fail("bracket not balanced".into()));
        }
        Ok(MultiStmt { stmts: stmts })
    }
    fn stmt(&mut self) -> ParseResult<Statement> {
        if self.consume_expect(|c| c.is_token_parts(), IF).is_some() {
            return Ok(Statement::If(self.if_()?));
        }
        if self.consume_expect(|c| c.is_token_parts(), FOR).is_some() {
            return Ok(Statement::For(self.for_()?));
        }
        if self.consume_expect(|c| c.is_token_parts(), WHILE).is_some() {
            return Ok(Statement::While(self.while_()?));
        }
        if self.consume("{").is_some() {
            return Ok(Statement::MStmt(self.multi_stmts()?));
        }
        let expr = self.expr()?;
        if self.consume(";").is_none() {
            return Err(self.fail("; expected".into()));
        }
        return Ok(Statement::Stmt(Stmt { expr }));
    }
    fn program(&mut self) -> ParseResult<Program> {
        let mut stmts = Vec::new();
        loop {
            if self.empty() {
                break;
            }
            match self.stmt() {
                Ok(s) => {
                    stmts.push(s);
                }
                Err(s) => return Err(s),
            }
        }
        Ok(Program {
            stmt: stmts,
            required_memory: self.ident_count * IDENTITY_OFFSET,
        })
    }
    fn parse(&mut self) -> ParseResult<Program> {
        self.program()
    }
}
pub fn parse(input: &String) -> ParseResult<Program> {
    Parser {
        input: input,
        index: 0,
        ident_count: 0,
        line_index: 0,
        idents: HashMap::new(),
        read_lines: 0,
    }
    .parse()
}
