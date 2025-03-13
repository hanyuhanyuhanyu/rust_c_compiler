use super::node::{
    Add, AddSub, Assign, Compare, Equality, Equals, Expr, Mul, MulDiv, Num, Primary, Program,
    Relational, Stmt, Unary,
};

#[derive(Debug)]
pub struct ParseFailure {
    pub index: usize,
    pub reason: String,
}
pub type ParseResult<T> = Result<T, ParseFailure>;

pub struct Parser<'a> {
    pub index: usize,
    pub input: &'a String,
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
                    num: None,
                    exp: Some(Box::new(e)),
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
            num: Some(Num { raw_num: raw_num }),
            exp: None,
        })
    }
    fn primary(&mut self, ope: Option<AddSub>) -> ParseResult<Primary> {
        if self.empty() {
            return Err(self.fail("number or ( expected".into()));
        }
        if self.consume("(") {
            self.p_exp(ope)
        } else {
            self.p_num(ope)
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
            Ok(p) => Ok(Unary { ope: ope, node: p }),
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
                equality: eq.unwrap(),
                assign: None,
            });
        }
        match self.assign() {
            Ok(a) => Ok(Assign {
                equality: eq.unwrap(),
                assign: Some(Box::new(a)),
            }),
            Err(e) => Err(e),
        }
    }
    pub fn expr(&mut self) -> ParseResult<Expr> {
        match self.assign() {
            Ok(a) => Ok(Expr { assign: a }),
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
        if self.consume(";") {
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
        Ok(Program { stmt: stmts })
    }
}
