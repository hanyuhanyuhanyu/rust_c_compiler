pub enum AddSub {
    Plus,
    Minus,
}
pub enum MulDiv {
    Multi,
    Divide,
}
pub enum Compare {
    Lt,
    Lte,
    Gt,
    Gte,
}
pub enum Equals {
    Equal,
    NotEqual,
}
pub struct Expr {
    pub node: Equality,
}
pub struct Equality {
    pub first: Relational,
    pub relationals: Vec<Relational>,
}

pub struct Relational {
    pub first: Add,
    pub ope: Option<Equals>,
    pub adds: Vec<Add>,
}
pub struct Add {
    pub first: Mul,
    pub ope: Option<Compare>,
    pub muls: Vec<Mul>,
}
pub struct Mul {
    pub ope: Option<AddSub>,
    pub first: Unary,
    pub unarys: Vec<Unary>,
}
pub struct Unary {
    pub ope: Option<MulDiv>,
    pub node: Primary,
}
pub struct Primary {
    pub ope: Option<AddSub>,
    pub num: Option<Num>,
    pub exp: Option<Expr>,
}
pub struct Num {
    pub raw_num: String,
}
trait PrimaryNode {}
impl PrimaryNode for Num {}
impl PrimaryNode for Expr {}
pub trait Node {}

impl Node for Equality {}
impl Node for Relational {}
impl Node for Add {}
impl Node for Mul {}
impl Node for Unary {}
impl Node for Primary {}
impl Node for Node {}
