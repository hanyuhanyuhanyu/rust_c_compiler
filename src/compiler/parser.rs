use std::collections::HashMap;

use super::node::{
    Add, AddSub, Assign, Compare, Equality, Equals, Expr, Ident, Lvar, Mul, MulDiv, Primary,
    PrimaryNode, Program, Relational, Stmt, Unary,
};
const IDENTITY_OFFSET: usize = 8;
const RETURN: &str = "return";

#[derive(Debug)]
pub struct ParseFailure {
    pub index: usize,
    pub reason: String,
}
pub type ParseResult<T> = Result<T, ParseFailure>;

pub struct Parser<'a> {
    pub index: usize,
    pub input: &'a String,
    pub ident_count: usize,
    pub idents: HashMap<String, usize>,
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
    fn consume(&mut self, str: &str) -> bool {
        self.space();
        if self
            .input
            .chars()
            .skip(self.index)
            .take(str.len())
            .eq(str.chars())
        {
            self.index += str.len();
            return true;
        }
        false
    }
    fn empty(&mut self) -> bool {
        self.space();
        self.index == self.input.len()
    }
    fn fail(&self, reason: String) -> ParseFailure {
        ParseFailure {
            index: self.index,
            reason: reason,
        }
    }
    fn check_top(&mut self, checker: fn(check: char) -> bool) -> bool {
        self.space();
        match self.input.chars().nth(self.index) {
            None => false,
            Some(c) => checker(c),
        }
    }
    fn seek(&mut self, checker: fn(check: char) -> bool) -> Option<char> {
        self.space();
        match self.input.chars().nth(self.index) {
            None => None,
            Some(c) => {
                if checker(c) {
                    self.index += 1;
                    Some(c)
                } else {
                    None
                }
            }
        }
    }
    fn will_return(&mut self) -> bool {
        self.space();
        if !self
            .input
            .chars()
            .skip(self.index)
            .take(RETURN.len())
            .eq(RETURN.chars())
        {
            return false;
        }
        match self.input.chars().nth(self.index + RETURN.len()) {
            None => false,
            Some(c) => {
                if c.is_token_parts() {
                    return false;
                }
                self.index += RETURN.len();
                true
            }
        }
    }
    fn extract(&mut self, checker: fn(check: char) -> bool) -> Option<String> {
        self.space();
        let mut str = "".to_string();
        loop {
            if self.empty() {
                break;
            }
            match self.seek(checker) {
                None => break,
                Some(c) => {
                    str.push(c);
                }
            };
        }
        if str.is_empty() { None } else { Some(str) }
    }

    fn space(&mut self) {
        loop {
            match self.input.chars().nth(self.index) {
                Some(' ') | Some('\r') | Some('\n') => {
                    self.index += 1;
                }
                _ => break,
            }
        }
    }
    fn p_exp(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        match self.expr() {
            Err(e) => Err(e),
            Ok(e) => {
                if !self.consume(")") {
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
            match self.seek(|c| c.is_numeric()) {
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
        match self.extract(|c| c.is_token_parts()) {
            None => Err(self.fail("identity can only contain 'a-z', 'A-Z', '0-9' and '_'".into())),

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
        if self.consume("(") {
            return self.p_exp(ope);
        }
        if self.check_top(|c| c.is_numeric()) {
            self.p_num(ope)
        } else {
            self.p_ident(ope)
        }
    }
    fn unary(&mut self, ope: Option<MulDiv>) -> ParseResult<Unary> {
        if self.empty() {
            return Err(self.fail("+, -, num or expression expected".into()));
        }
        let addsub = if self.consume("+") {
            Some(AddSub::Plus)
        } else if self.consume("-") {
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
            let ope = if self.consume("*") {
                Some(MulDiv::Multi)
            } else if self.consume("/") {
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
            let ope = if self.consume("+") {
                Some(AddSub::Plus)
            } else if self.consume("-") {
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
            let ope = if self.consume(">=") {
                Some(Compare::Gte)
            } else if self.consume("<=") {
                Some(Compare::Lte)
            } else if self.consume("<") {
                Some(Compare::Lt)
            } else if self.consume(">") {
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
            let ope = if self.consume("==") {
                Some(Equals::Equal)
            } else if self.consume("!=") {
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
        if !self.consume("=") {
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
        let ret = self.will_return();
        match self.assign() {
            Ok(a) => Ok(Expr {
                assign: a,
                ret: ret,
            }),
            Err(e) => Err(e),
        }
    }
    fn stmt(&mut self) -> ParseResult<Stmt> {
        let stmt = match self.expr() {
            Ok(e) => Ok(Stmt { expr: e }),
            Err(e) => Err(e),
        };
        if stmt.is_err() {
            return stmt;
        }
        if !self.consume(";") {
            return Err(self.fail("; expected".into()));
        }
        return stmt;
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
    pub fn parse(&mut self) -> ParseResult<Program> {
        self.program()
    }
}
