use std::collections::HashMap;

use super::{
    consts::{
        BLOCK_EXPECTED, BRACKET_NOT_BALANCED, FOR, IDENTITY_OFFSET, IDENTITY_WANTED, IF, INT,
        RETURN, TYPE_WANTED, TYPES, WHILE, sizeof,
    },
    node::{
        Add, AddSub, Asgn, Assign, Block, Compare, Equality, Equals, Expr, ExprAssign, Fcall, Fdef,
        For, Ident, If, Lvar, Mul, MulDiv, Primary, PrimaryNode, Program, PtrOpe, Relational, Rvar,
        Statement, Stmt, Type, Typed, Unary, UnaryPtr, UnaryVar, VarDef, While,
    },
};
const DEBUG: bool = false;
#[derive(Debug)]
pub struct ParseFailure {
    pub index: usize,
    pub read_line: usize,
    pub source: Option<String>,
    pub reason: String,
}
pub type ParseResult<T> = Result<T, ParseFailure>;

#[derive(Debug)]
pub struct Parser<'a> {
    pub index: usize,
    pub input: &'a String,
    pub required_memory: usize,
    pub funcs: HashMap<String, Type>,
    pub idents: HashMap<String, VarDef>,
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
fn type_match(_a: &Type, _b: &Type) -> bool {
    // 本来は演算の種類ごとに計算可能性を考える必要があるのでこれ一つの関数で処理することはできない
    // だけど一旦はこれでよし
    return true;
    // if a == b {
    //     return true;
    // }
    // match a {
    //     Type::_Panic => false,
    //     Type::Int => match b {
    //         Type::Int | Type::LInt => true,
    //         _ => false,
    //     },
    //     Type::LInt => type_match(&Type::Int, b),
    //     Type::Ptr(aa) => match b {
    //         Type::Ptr(bb) => type_match(aa, bb),
    //         _ => false,
    //     },
    // }
}
impl Parser<'_> {
    fn dbg(&self, note: String) {
        if DEBUG {
            if note.len() > 0 {
                print!("{}/", note);
            }
            println!("{:?}, {}", self.input.chars().nth(self.index), self.index);
            println!("\t{}\n\t{}^", self.input, " ".repeat(self.index));
        }
    }
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
        self.dbg(format!("fail {}", reason));
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
    fn p_exp(&mut self, ope: Option<AddSub>) -> ParseResult<Typed<Primary>> {
        let node = self.parenthesized(|p| p.expr())?;
        let type_ = node.1.clone();
        return Ok((
            Primary {
                ope,
                node: (PrimaryNode::Expr(Box::new(node.0)), type_.clone()),
            },
            type_,
        ));
    }
    fn p_num(&mut self, ope: Option<AddSub>) -> ParseResult<Typed<Primary>> {
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
        Ok((
            Primary {
                ope: ope,
                node: (PrimaryNode::Num((raw_num, Type::LInt)), Type::LInt),
            },
            Type::LInt,
        ))
    }
    fn get_ident(&mut self) -> Option<String> {
        let first = self.top_f(|c| c.is_token_first())?;
        let tail = self.consume_f(|c| c.is_token_parts()).unwrap_or("".into());
        Some(format!("{}{}", first, tail))
    }
    fn p_ident(&mut self, ope: Option<AddSub>, ident: String) -> ParseResult<Typed<Primary>> {
        let var = self.idents.get(&ident);
        if var.is_none() {
            return Err(self.fail(format!("var {} undeclared", ident)));
        }
        let v = var.unwrap();
        Ok((
            Primary {
                ope,
                node: (
                    PrimaryNode::Lv(Lvar::Id(Ident {
                        _type_: v._type_.clone(),
                        offset: v.offset,
                        // refable, refで剥がして良い回数を持ちたい
                    })),
                    v._type_.clone(),
                ),
            },
            v._type_.clone(),
        ))
    }
    fn for_test_is_func_available(&self, ident: String) -> bool {
        ident == "_p"
    }
    fn fcall(&mut self, ope: Option<AddSub>, ident: String) -> ParseResult<Typed<Primary>> {
        self.dbg("fcall".into());
        let f = self.funcs.get(&ident);
        let available = self.for_test_is_func_available(ident.clone());
        if !available && f.is_none() {
            return Err(self.fail(format!("func {} is undefined", &ident)));
        }
        let type_ = if available {
            Type::Int
        } else {
            f.unwrap().clone()
        };
        let args = self.parenthesized(|p| {
            p.loop_while(
                |p, _| !p.check_top(")") && !p.empty(),
                |p, _| p.consume(",").is_some(),
                |p, _| p.expr(),
            )
        })?;
        // TODO: argsの型チェックもしようね
        Ok((
            Primary {
                ope,
                node: (PrimaryNode::Fcall(Fcall { ident, args }), type_.clone()),
            },
            type_,
        ))
    }
    fn primary(&mut self, ope: Option<AddSub>) -> ParseResult<Typed<Primary>> {
        self.dbg("primary".into());
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
    fn unary(&mut self, ope: Option<MulDiv>) -> ParseResult<Typed<Unary>> {
        self.dbg("unary".into());
        if self.empty() {
            return Err(self.fail("+, -, num or expression expected".into()));
        }
        if self.consume("*").is_some() {
            let unary = self.unary(ope)?;
            let t = unary.1.clone();
            return match t {
                Type::Ptr(inside) => Ok((
                    Unary::Ptr(UnaryPtr {
                        ope: PtrOpe::Ref,
                        unary: Box::new(unary),
                    }),
                    *inside,
                )),
                ty => Err(self.fail(format!("cannot get ref of type {:?}", ty))),
            };
        } else if self.consume("&").is_some() {
            let unary = self.unary(ope)?;
            let t = unary.1.clone();
            return Ok((
                Unary::Ptr(UnaryPtr {
                    ope: PtrOpe::Deref,
                    unary: Box::new(unary),
                }),
                Type::Ptr(Box::new(t)),
            ));
        }
        let addsub = if self.consume("+").is_some() {
            Some(AddSub::Plus)
        } else if self.consume("-").is_some() {
            Some(AddSub::Minus)
        } else {
            None
        };
        let prim = self.primary(addsub)?;
        let t = prim.1.clone();
        Ok((Unary::Var(UnaryVar { ope, prim: prim }), t))
    }
    fn mul(&mut self, ope: Option<AddSub>) -> ParseResult<Typed<Mul>> {
        self.dbg("mul".into());
        // 一般化したい
        let (una, type_) = self.unary(None)?;
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
        Ok((
            Mul {
                first: (una, type_.clone()),
                ope,
                unarys,
            },
            type_,
        ))
    }
    fn add(&mut self, ope: Option<Compare>) -> ParseResult<Typed<Add>> {
        self.dbg("add".into());
        let (first, type_) = self.mul(None)?;
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

        if muls.len() == 0 {
            return Ok((
                Add {
                    first: (first, type_.clone()),
                    ope,
                    muls,
                },
                type_,
            ));
        }
        if !type_match(&type_, &muls.first().unwrap().1) {
            Err(self.fail(format!(
                "bad operator usage {:?} {:?} {:?}",
                type_,
                muls.first().unwrap().0.ope,
                muls.first().unwrap().1
            )))
        } else {
            Ok((
                Add {
                    first: (first, type_.clone()),
                    ope,
                    muls,
                },
                type_,
            ))
        }
    }
    fn relational(&mut self, ope: Option<Equals>) -> ParseResult<Typed<Relational>> {
        self.dbg("relational".into());
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
        if adds.len() == 0 {
            let t = first.1.clone();
            Ok((Relational { first, ope, adds }, t))
        } else {
            if adds.first().is_none() {
                return Err(self.fail("compiler bug, relational.add accidentally empty".into()));
            }
            let un = adds.first().unwrap();
            if !type_match(&un.1, &first.1) {
                Err(self.fail(format!(
                    "bad operator usage {:?} {:?} {:?}",
                    first.1, un.0.ope, un.1
                )))
            } else {
                Ok((
                    Relational { first, ope, adds },
                    Type::Int, // TODO bool
                ))
            }
        }
    }
    fn equality(&mut self) -> ParseResult<Typed<Equality>> {
        self.dbg("equality".into());
        let (first, l_type) = self.relational(None)?;
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
        if relationals.first().is_none() {
            Ok((
                Equality {
                    first: (first, l_type.clone()),
                    relationals,
                },
                l_type,
            ))
        } else {
            if relationals.first().unwrap().1 != l_type {
                Err(self.fail(format!(
                    "bad operation usage {:?} {:?} {:?}",
                    l_type,
                    relationals.first().unwrap().0.ope,
                    relationals.first().unwrap().1
                )))
            } else {
                Ok((
                    Equality {
                        first: (first, l_type),
                        relationals,
                    },
                    Type::Int,
                )) // boolにしたい
            }
        }
    }
    fn rvar(&mut self) -> ParseResult<Typed<Equality>> {
        self.dbg("rvar".into());
        self.equality()
    }
    fn assign(&mut self) -> ParseResult<Typed<Assign>> {
        self.dbg("assign".into());
        let (eq, rtype) = self.rvar()?;
        let lvar = eq.lvar();
        if lvar.is_none() || self.consume("=").is_none() {
            return Ok((
                Assign::Rv(Rvar {
                    eq: (eq, rtype.clone()),
                }),
                rtype,
            ));
        }
        Ok((
            Assign::Asgn(Asgn {
                lvar: (eq, rtype.clone()),
                rvar: Box::new(self.expr()?),
            }),
            rtype,
        ))
    }
    fn lvar(&mut self) -> ParseResult<(usize, String)> {
        self.dbg("lvar".into());
        let ref_count = self.consume_f(|c| c == '*').unwrap_or("".into()).len();
        let next_token = self.get_ident();
        if next_token.is_none() {
            return Err(self.fail(IDENTITY_WANTED.into()));
        }
        Ok((ref_count, next_token.unwrap()))
    }
    fn gen_type(&mut self, t: Type, ref_count: usize) -> Type {
        if ref_count == 0 {
            t
        } else {
            Type::Ptr(Box::new(self.gen_type(t, ref_count - 1)))
        }
    }
    fn def(&mut self) -> ParseResult<(Vec<VarDef>, Option<Assign>, Type)> {
        self.dbg("def".into());
        let type_ = self.find_type();
        if type_.is_none() {
            return Err(self.fail("type expected".into()));
        }
        let vardefs = self.loop_while(
            |p, _| p.check_top_f(|c| c.is_token_first() || c == '*') && !p.empty(),
            |p, _| p.consume(",").is_some(),
            |p, _| {
                let (ref_count, ident) = p.lvar()?;
                if p.idents.get(&ident).is_some() {
                    return Err(p.fail(format!("multi definition for {}", ident)));
                }
                let type_ = p.gen_type(type_.clone().unwrap(), ref_count);
                let memory = sizeof(&type_);
                p.required_memory += memory;
                let def = VarDef {
                    ident: ident.clone(),
                    offset: p.required_memory,
                    _type_: type_,
                    _ref_count_: ref_count,
                };
                p.idents.insert(ident.clone(), def.clone());
                Ok(def)
            },
        )?;
        if self.consume("=").is_none() {
            return Ok((vardefs, None, type_.unwrap()));
        }
        Ok((vardefs, Some(self.assign()?.0), type_.unwrap())) // TODO 本当に良い？
    }
    fn expr(&mut self) -> ParseResult<Typed<Expr>> {
        self.dbg("expr".into());
        if self.check_type() {
            let (a, b, type_) = self.def()?;
            return Ok((Expr::VarAsgn(a, b), type_)); // FIXME: このtypeはint *xとしたときのint部分しか見ておらず、ポインタの部分は見られていないので、各変数の正しい型を見られていない。Generator側でうまく回避しているが、きちんとした形を模索すべき
        }
        let ret = self.consume_expect(|c| c.is_token_parts(), RETURN);
        let assign = self.assign()?;
        Ok((
            Expr::Asgn(ExprAssign {
                assign: assign.0,
                ret: ret.is_some(),
            }),
            assign.1,
        ))
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
        self.dbg("block".into());
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
        self.dbg("stmt".into());
        if self.consume(";").is_some() {
            return Ok(Statement::Nothing);
        }
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
    fn check_type(&mut self) -> bool {
        TYPES.iter().any(|t| self.check_top(t))
    }
    fn find_type(&mut self) -> Option<Type> {
        let ty = TYPES.iter().fold(None, |acc, cur| match acc {
            None => self.consume(cur),
            Some(some) => Some(some),
        })?;
        match ty.as_str() {
            INT => Some(Type::Int),
            _ => None,
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
        self.dbg("par".into());
        if self.consume("(").is_none() {
            return Err(self.fail("parenthesis expected".into()));
        }
        let ret = f(self);
        if self.consume(")").is_none() {
            return Err(self.fail("parenthesis unbalanced".into()));
        }
        self.dbg("par end".into());
        ret
    }
    fn args(&mut self) -> ParseResult<Vec<VarDef>> {
        self.dbg("args".into());
        self.parenthesized(|p| {
            p.loop_while(
                |p, _| !p.check_top(")") && !p.empty(),
                |p, _| p.consume(",").is_some(),
                |p, count| {
                    let type_ = p.find_type();
                    if type_.is_none() {
                        return Err(p.fail(TYPE_WANTED.into()));
                    }
                    let (ref_count, ident) = p.lvar()?;

                    Ok(VarDef {
                        ident: ident,
                        _type_: type_.unwrap(),
                        _ref_count_: ref_count,
                        offset: (count + 1) * IDENTITY_OFFSET, // TODO 適切な大きさで確保する
                    })
                },
            )
        })
    }

    fn fdef(&mut self) -> ParseResult<Fdef> {
        self.dbg("fdef".into());
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
            idents.insert(arg.ident.clone(), arg.clone());
        }
        self.funcs.insert(ident.clone().unwrap(), type_.unwrap());
        let mut child = Parser {
            index: self.index,
            input: self.input,
            funcs: self.funcs.clone(),
            required_memory: args.last().map_or(0, |v| v.offset),
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
            required_memory: child.required_memory,
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
        required_memory: 0,
        line_index: 0,
        funcs: HashMap::new(),
        idents: HashMap::new(),
        read_lines: 0,
    }
    .parse()
}
