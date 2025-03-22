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
#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Ptr(()),
}
#[derive(Debug)]
pub struct Fcall {
    pub ident: String,
    pub args: Vec<Expr>,
}
#[derive(Debug)]
pub struct Fdef {
    // pub type_: Type,
    pub ident: String,
    pub fimpl: Block,
    pub args: Vec<VarDef>,
    pub required_memory: usize,
}
#[derive(Debug)]
pub struct Program {
    pub fdefs: Vec<Fdef>,
    // pub stmt: Vec<Statement>,
    // pub required_memory: usize,
}

#[derive(Debug)]
pub struct If {
    pub cond: Expr,
    pub stmt: Box<Statement>,
    pub else_: Option<Box<Statement>>,
}
#[derive(Debug)]
pub struct For {
    pub init: Option<Expr>,
    pub cond: Option<Expr>,
    pub step: Option<Expr>,
    pub stmt: Box<Statement>,
}
#[derive(Debug)]
pub struct While {
    pub cond: Expr,
    pub stmt: Box<Statement>,
}
#[derive(Debug)]
pub struct Stmt {
    pub expr: Expr,
}
#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Statement>,
}
#[derive(Debug)]
pub enum Statement {
    If(If),
    For(For),
    While(While),
    Stmt(Stmt),
    MStmt(Block),
    Nothing,
}
#[derive(Debug)]
pub enum Expr {
    Asgn(ExprAssign),
    VarAsgn(Vec<VarDef>, Option<Assign>),
}
impl Expr {
    pub fn does_return(&self) -> bool {
        match self {
            Expr::Asgn(e) => e.ret,
            _ => false,
        }
    }
}
#[derive(Debug)]
pub struct ExprAssign {
    pub assign: Assign,
    pub ret: bool,
}
#[derive(Debug)]
pub struct Rvar {
    pub eq: Equality,
}
#[derive(Debug)]
pub struct Asgn {
    pub lvar: Equality,
    pub rvar: Box<Expr>,
}
#[derive(Debug, Clone)]
pub struct VarDef {
    pub ident: String,
    pub _type_: Type,
    pub _ref_count_: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub enum Assign {
    Rv(Rvar),
    Asgn(Asgn),
}
#[derive(Debug)]
pub struct Equality {
    pub first: Relational,
    pub relationals: Vec<Relational>,
}
impl Equality {
    pub fn lvar(&self) -> Option<(&Lvar, usize)> {
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
    fn lvar(&self) -> Option<(&Lvar, usize)> {
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
    fn lvar(&self) -> Option<(&Lvar, usize)> {
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
    fn lvar(&self) -> Option<(&Lvar, usize)> {
        if self.ope.is_some() || self.unarys.len() > 0 {
            return None;
        }
        self.first.lvar(0)
    }
}
#[derive(Debug)]
pub enum PtrOpe {
    Ref,
    Deref,
}
#[derive(Debug)]
pub struct UnaryPtr {
    pub ope: PtrOpe,
    pub unary: Box<Unary>,
}
#[derive(Debug)]
pub struct UnaryVar {
    pub ope: Option<MulDiv>,
    pub prim: Primary,
}
#[derive(Debug)]
pub enum Unary {
    Ptr(UnaryPtr),
    Var(UnaryVar),
}
impl Unary {
    fn lvar(&self, ref_count: usize) -> Option<(&Lvar, usize)> {
        match self {
            Unary::Var(p) => p.prim.lvar(ref_count),
            Unary::Ptr(p) => match p.ope {
                PtrOpe::Deref => None,
                PtrOpe::Ref => p.unary.lvar(ref_count + 1),
            },
        }
    }
    pub fn ope(&self) -> &Option<MulDiv> {
        match self {
            Unary::Ptr(p) => p.unary.ope(),
            Unary::Var(p) => &p.ope,
        }
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
    Fcall(Fcall),
}
#[derive(Debug)]
pub struct Primary {
    pub ope: Option<AddSub>,
    pub node: PrimaryNode,
}
#[derive(Debug)]
pub struct Ident {
    // pub type_: Type,
    pub offset: usize,
}
impl Primary {
    fn lvar(&self, ref_count: usize) -> Option<(&Lvar, usize)> {
        if self.ope.is_some() {
            return None;
        }
        match &self.node {
            PrimaryNode::Lv(l) => Some((l, ref_count)),
            _ => None,
        }
    }
}
