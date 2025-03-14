#[derive(Debug)]
pub enum AddSub {
    Plus,
    Minus,
}
#[derive(Debug)]
pub enum MulDiv {
    Multi,
    Divide,
}
#[derive(Debug)]
pub enum Compare {
    Lt,
    Lte,
    Gt,
    Gte,
}
#[derive(Debug)]
pub enum Equals {
    Equal,
    NotEqual,
}
#[derive(Debug)]
pub struct Program {
    pub stmt: Vec<Stmt>,
    pub required_memory: usize,
}

#[derive(Debug)]
pub struct Stmt {
    pub expr: Expr,
}
#[derive(Debug)]
pub struct Expr {
    pub assign: Assign,
    pub ret: bool,
}
#[derive(Debug)]
pub struct Assign {
    pub lvar: Equality,
    pub rvar: Option<Box<Assign>>,
}
#[derive(Debug)]
pub struct Equality {
    pub first: Relational,
    pub relationals: Vec<Relational>,
}
impl Equality {
    pub fn lvar(&self) -> Option<&Lvar> {
        if self.relationals.len() > 0 {
            return None;
        }
        self.first.lvar()
    }
}
#[derive(Debug)]
pub struct Relational {
    pub first: Add,
    pub ope: Option<Equals>,
    pub adds: Vec<Add>,
}
impl Relational {
    fn lvar(&self) -> Option<&Lvar> {
        if self.ope.is_some() || self.adds.len() > 0 {
            return None;
        }
        self.first.lvar()
    }
}
#[derive(Debug)]
pub struct Add {
    pub first: Mul,
    pub ope: Option<Compare>,
    pub muls: Vec<Mul>,
}
impl Add {
    fn lvar(&self) -> Option<&Lvar> {
        if self.ope.is_some() || self.muls.len() > 0 {
            return None;
        }
        self.first.lvar()
    }
}
#[derive(Debug)]
pub struct Mul {
    pub first: Unary,
    pub ope: Option<AddSub>,
    pub unarys: Vec<Unary>,
}
impl Mul {
    fn lvar(&self) -> Option<&Lvar> {
        if self.ope.is_some() || self.unarys.len() > 0 {
            return None;
        }
        self.first.lvar()
    }
}
#[derive(Debug)]
pub struct Unary {
    pub ope: Option<MulDiv>,
    pub prim: Primary,
}
impl Unary {
    fn lvar(&self) -> Option<&Lvar> {
        if self.ope.is_some() {
            return None;
        }
        self.prim.lvar()
    }
}
#[derive(Debug)]
pub enum Lvar {
    Id(Ident),
}
#[derive(Debug)]
pub enum PrimaryNode {
    Num(String),
    Lv(Lvar),
    Expr(Box<Expr>),
}
#[derive(Debug)]
pub struct Primary {
    pub ope: Option<AddSub>,
    pub node: PrimaryNode,
}
#[derive(Debug)]
pub struct Ident {
    pub offset: usize,
}
impl Primary {
    fn lvar(&self) -> Option<&Lvar> {
        if self.ope.is_some() {
            return None;
        }
        match &self.node {
            PrimaryNode::Lv(l) => Some(l),
            _ => None,
        }
    }
}
