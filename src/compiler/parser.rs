use std::collections::HashMap;

use super::{
    consts::{
        BLOCK_EXPECTED, BRACKET_NOT_BALANCED, FOR, IDENTITY_OFFSET, IDENTITY_WANTED, IF, RETURN,
        TYPE_WANTED, TYPES, WHILE,
    },
    node::{
        Add, AddSub, Arg, Asgn, Assign, Block, Compare, Equality, Equals, Expr, Fcall, Fdef, For,
        Ident, If, Lvar, Mul, MulDiv, Primary, PrimaryNode, Program, PtrOpe, Relational, Rvar,
        Statement, Stmt, Type, Unary, UnaryPtr, UnaryVar, While,
    },
};
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
    fn is_token_first(self) -> bool;
}
impl IsToken for char {
    fn is_token_first(self) -> bool {
        self.is_ascii_alphabetic() || self == '_'
    }
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
        Ok(Primary {
            ope,
            node: PrimaryNode::Expr(Box::new(self.parenthesized(|p| p.expr())?)),
        })
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
    fn get_ident(&mut self) -> Option<String> {
        let first = self.top_f(|c| c.is_token_first())?;
        let tail = self.consume_f(|c| c.is_token_parts()).unwrap_or("".into());
        Some(format!("{}{}", first, tail))
    }
    fn p_ident(&mut self, ope: Option<AddSub>, ident: String) -> ParseResult<Primary> {
        let offset = match self.idents.get(&ident) {
            None => {
                self.ident_count += 1;
                let o = self.ident_count * IDENTITY_OFFSET;
                self.idents.insert(ident, o);
                o
            }
            Some(ofs) => *ofs,
        };

        Ok(Primary {
            ope,
            node: PrimaryNode::Lv(Lvar::Id(Ident {
                // type_: Type::Int,
                offset: offset,
            })),
        })
    }
    fn fcall(&mut self, ope: Option<AddSub>, ident: String) -> ParseResult<Primary> {
        let args = self.parenthesized(|p| {
            p.loop_while(
                |p, _| !p.check_top(")") && !p.empty(),
                |p, _| p.consume(",").is_some(),
                |p, _| p.expr(),
            )
        })?;
        Ok(Primary {
            ope,
            node: PrimaryNode::Fcall(Fcall { ident, args }),
        })
    }
    fn primary(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        if self.empty() {
            return Err(self.fail("number or ( expected".into()));
        }
        if self.check_top("(") {
            return self.p_exp(ope);
        }
        // 0-9なら数値と決めつけてよいかは疑問の余地あり
        if self.check_top_f(|c| c.is_numeric()) {
            return self.p_num(ope);
        }
        let ident = self.consume_f(|c| c.is_token_parts());
        if ident.is_none() {
            return Err(self.fail("identity expected".into()));
        }
        if self.check_top("(") {
            self.fcall(ope, ident.unwrap())
        } else {
            self.p_ident(ope, ident.unwrap())
        }
    }
    fn unary(&mut self, ope: Option<MulDiv>) -> ParseResult<Unary> {
        if self.empty() {
            return Err(self.fail("+, -, num or expression expected".into()));
        }
        if self.consume("*").is_some() {
            return Ok(Unary::Ptr(UnaryPtr {
                ope: PtrOpe::Ref,
                unary: Box::new(self.unary(ope)?),
            }));
        } else if self.consume("&").is_some() {
            return Ok(Unary::Ptr(UnaryPtr {
                ope: PtrOpe::Deref,
                unary: Box::new(self.unary(ope)?),
            }));
        }
        let addsub = if self.consume("+").is_some() {
            Some(AddSub::Plus)
        } else if self.consume("-").is_some() {
            Some(AddSub::Minus)
        } else {
            None
        };
        Ok(Unary::Var(UnaryVar {
            ope,
            prim: self.primary(addsub)?,
        }))
    }
    fn mul(&mut self, ope: Option<AddSub>) -> ParseResult<Mul> {
        // 一般化したい
        let first = self.unary(None)?;
        let unarys = self.loop_while(
            |p, _| !p.empty() && (p.check_top("/") || p.check_top("*")),
            |_, _| true,
            |p, _| {
                let ope = if p.consume("*").is_some() {
                    Some(MulDiv::Multi)
                } else if p.consume("/").is_some() {
                    Some(MulDiv::Divide)
                } else {
                    return Err(p.fail("compiler bug, * or / must be here".into()));
                };
                p.unary(ope)
            },
        )?;
        Ok(Mul { first, ope, unarys })
    }
    fn add(&mut self, ope: Option<Compare>) -> ParseResult<Add> {
        let first = self.mul(None)?;
        let muls = self.loop_while(
            |p, _| !p.empty() && (p.check_top("+") || p.check_top("-")),
            |_, _| true,
            |p, _| {
                let ope = if p.consume("+").is_some() {
                    Some(AddSub::Plus)
                } else if p.consume("-").is_some() {
                    Some(AddSub::Minus)
                } else {
                    return Err(p.fail("compiler bug, + or - must be here".into()));
                };
                p.mul(ope)
            },
        )?;
        Ok(Add {
            first,
            ope: ope,
            muls,
        })
    }
    fn relational(&mut self, ope: Option<Equals>) -> ParseResult<Relational> {
        let first = self.add(None)?;
        let checker = |p: &mut Self, _| {
            !p.empty()
                && (p.check_top(">=") || p.check_top("<=") || p.check_top(">") || p.check_top("<"))
        };
        let adds = self.loop_while(
            checker,
            |_, _| true,
            |p, _| {
                let ope = if p.consume(">=").is_some() {
                    Some(Compare::Gte)
                } else if p.consume("<=").is_some() {
                    Some(Compare::Lte)
                } else if p.consume("<").is_some() {
                    Some(Compare::Lt)
                } else if p.consume(">").is_some() {
                    Some(Compare::Gt)
                } else {
                    return Err(p.fail("compiler bug, >=  or <= or > or < must be here".into()));
                };
                p.add(ope)
            },
        )?;

        Ok(Relational { first, ope, adds })
    }
    fn equality(&mut self) -> ParseResult<Equality> {
        let first = self.relational(None)?;
        let checker = |p: &mut Self, _| !p.empty() && (p.check_top("==") || p.check_top("!="));
        let relationals = self.loop_while(
            checker,
            |_, _| true,
            |p, _| {
                let ope = if p.consume("==").is_some() {
                    Some(Equals::Equal)
                } else if p.consume("!=").is_some() {
                    Some(Equals::NotEqual)
                } else {
                    return Err(p.fail("compiler bug, >=  or <= or > or < must be here".into()));
                };
                p.relational(ope)
            },
        )?;
        Ok(Equality { first, relationals })
    }
    fn rvar(&mut self) -> ParseResult<Equality> {
        self.equality()
    }
    fn assign(&mut self) -> ParseResult<Assign> {
        let eq = self.rvar()?;
        let lvar = eq.lvar();
        if lvar.is_none() || self.consume("=").is_none() {
            return Ok(Assign::Rv(Rvar { eq }));
        }
        Ok(Assign::Asgn(Asgn {
            lvar: eq,
            rvar: Box::new(self.expr()?),
        }))
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
        let cond = self.parenthesized(|p| p.expr())?;
        let stmt = self.stmt()?;
        Ok(While {
            cond,
            stmt: Box::new(stmt),
        })
    }
    fn for_(&mut self) -> ParseResult<For> {
        if self.consume("(").is_none() {
            return Err(self.fail("( expected after 'for'".into()));
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
        let cond = self.parenthesized(|p| p.expr())?;
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
    fn block(&mut self) -> ParseResult<Block> {
        if self.consume("{").is_none() {
            return Err(self.fail(BLOCK_EXPECTED.into()));
        }
        let mut stmts = Vec::new();
        loop {
            if self.check_top("}") || self.empty() {
                break;
            };
            stmts.push(self.stmt()?);
        }
        if self.consume("}").is_none() {
            return Err(self.fail(BRACKET_NOT_BALANCED.into()));
        }
        Ok(Block { stmts: stmts })
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
        if self.check_top("{") {
            return Ok(Statement::MStmt(self.block()?));
        }
        let expr = self.expr()?;
        if self.consume(";").is_none() {
            return Err(self.fail("; expected".into()));
        }
        return Ok(Statement::Stmt(Stmt { expr }));
    }
    fn find_type(&mut self) -> Option<Type> {
        let ty = TYPES.iter().fold(None, |acc, cur| match acc {
            None => self.consume(cur),
            Some(some) => Some(some),
        })?;
        match ty.as_str() {
            // INT => Some(Type::Int),
            // _ => None,
            _ => Some(Type::Int),
        }
    }
    fn loop_while<T>(
        &mut self,
        mut check_on_start: impl FnMut(&mut Self, usize) -> bool,
        mut check_on_end: impl FnMut(&mut Self, usize) -> bool,
        mut f: impl FnMut(&mut Self, usize) -> ParseResult<T>,
    ) -> ParseResult<Vec<T>> {
        let mut ret = Vec::new();
        let mut count = 0;
        loop {
            if !check_on_start(self, count) {
                break;
            }
            let n = f(self, count);
            ret.push(n?);
            if !check_on_end(self, count) {
                break;
            }
            count += 1
        }
        Ok(ret)
    }
    fn parenthesized<T>(
        &mut self,
        mut f: impl FnMut(&mut Self) -> ParseResult<T>,
    ) -> ParseResult<T> {
        if self.consume("(").is_none() {
            return Err(self.fail("parenthesis expected".into()));
        }
        let ret = f(self);
        if self.consume(")").is_none() {
            return Err(self.fail("parenthesis unbalanced".into()));
        }
        ret
    }
    fn args(&mut self) -> ParseResult<Vec<Arg>> {
        self.parenthesized(|p| {
            p.loop_while(
                |p, _| !p.check_top(")") && !p.empty(),
                |p, _| p.consume(",").is_some(),
                |p, count| {
                    let type_ = p.find_type();
                    if type_.is_none() {
                        return Err(p.fail(TYPE_WANTED.into()));
                    }
                    let ident = p.get_ident();
                    if ident.is_none() {
                        return Err(p.fail(IDENTITY_WANTED.into()));
                    }
                    Ok(Arg {
                        ident: ident.unwrap(),
                        offset: (count + 1) * IDENTITY_OFFSET,
                    })
                },
            )
        })
    }

    fn fdef(&mut self) -> ParseResult<Fdef> {
        let type_ = self.find_type();
        if type_.is_none() {
            return Err(self.fail(TYPE_WANTED.into()));
        }
        let ident = self.get_ident();
        if ident.is_none() {
            return Err(self.fail(IDENTITY_WANTED.into()));
        }
        let args = self.args()?;
        let mut idents = HashMap::new();
        for arg in args.iter() {
            idents.insert(arg.ident.clone(), arg.offset);
        }
        let mut child = Parser {
            index: self.index,
            input: self.input,
            ident_count: args.len(),
            idents,
            read_lines: self.read_lines,
            line_index: self.line_index,
        };
        let fimpl = child.block()?;
        self.index = child.index;
        self.read_lines = child.read_lines;
        self.line_index = child.line_index;

        Ok(Fdef {
            ident: ident.unwrap(),
            fimpl,
            args,
            required_memory: child.ident_count * IDENTITY_OFFSET,
        })
    }
    fn program(&mut self) -> ParseResult<Program> {
        let mut fdefs = Vec::new();
        loop {
            if self.empty() {
                break;
            }
            fdefs.push(self.fdef()?);
        }
        Ok(Program { fdefs: fdefs })
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
