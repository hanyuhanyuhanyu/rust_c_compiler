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
pub struct Expr {
    pub node: Equality,
}
#[derive(Debug)]
pub struct Equality {
    pub first: Relational,
    pub relationals: Vec<Relational>,
}
#[derive(Debug)]
pub struct Relational {
    pub first: Add,
    pub ope: Option<Equals>,
    pub adds: Vec<Add>,
}
#[derive(Debug)]
pub struct Add {
    pub first: Mul,
    pub ope: Option<Compare>,
    pub muls: Vec<Mul>,
}
#[derive(Debug)]
pub struct Mul {
    pub ope: Option<AddSub>,
    pub first: Unary,
    pub unarys: Vec<Unary>,
}
#[derive(Debug)]
pub struct Unary {
    pub ope: Option<MulDiv>,
    pub node: Primary,
}
#[derive(Debug)]
pub struct Primary {
    pub ope: Option<AddSub>,
    pub num: Option<Num>,
    pub exp: Option<Box<Expr>>,
}
#[derive(Debug)]
pub struct Num {
    pub raw_num: String,
}
