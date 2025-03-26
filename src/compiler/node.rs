use super::type_::Type;

pub type Typed<T> = (T, Type);
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
pub struct Fcall {
    pub ident: String,
    pub args: Vec<Typed<Expr>>,
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
    pub cond: (Expr, Type),
    pub stmt: Box<Statement>,
    pub else_: Option<Box<Statement>>,
}
#[derive(Debug)]
pub struct For {
    pub init: Option<Typed<Expr>>,
    pub cond: Option<Typed<Expr>>,
    pub step: Option<Typed<Expr>>,
    pub stmt: Box<Statement>,
}
#[derive(Debug)]
pub struct While {
    pub cond: Typed<Expr>,
    pub stmt: Box<Statement>,
}
#[derive(Debug)]
pub struct Stmt {
    pub expr: Typed<Expr>,
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
    pub eq: Typed<Equality>,
}
#[derive(Debug)]
pub struct Asgn {
    pub lvar: Typed<Equality>,
    pub rvar: Box<Typed<Expr>>,
}
#[derive(Debug, Clone)]
pub struct VarDef {
    pub ident: String,
    pub type_: Type,
    pub _ref_count_: usize,
    pub offset: usize,
}

#[derive(Debug)]
pub enum Assign {
    Rv(Rvar),
    Asgn(Asgn),
}
impl Assign {
    pub fn type_(&self) -> Type {
        match &self {
            Assign::Rv(r) => r.eq.1.clone(),
            Assign::Asgn(a) => a.lvar.1.clone(),
        }
    }
}
#[derive(Debug)]
pub struct Equality {
    pub first: Typed<Relational>,
    pub relationals: Vec<Typed<Relational>>,
}
impl Equality {
    pub fn is_lvar(&self) -> bool {
        self.first.0.is_lvar() && self.relationals.len() == 0
    }
}
#[derive(Debug)]
pub struct Relational {
    pub first: (Add, Type),
    pub ope: Option<Equals>,
    pub adds: Vec<(Add, Type)>,
}
impl Relational {
    pub fn is_lvar(&self) -> bool {
        if self.ope.is_some() || self.adds.len() > 0 {
            return false;
        }
        self.first.0.is_lvar()
    }
}
#[derive(Debug)]
pub struct Add {
    pub first: (Mul, Type),
    pub ope: Option<Compare>,
    pub muls: Vec<(Mul, Type)>,
}
impl Add {
    pub fn is_lvar(&self) -> bool {
        if self.ope.is_some() || self.muls.len() > 0 {
            return false;
        }
        self.first.0.is_lvar()
    }
}
#[derive(Debug)]
pub struct Mul {
    pub first: (Unary, Type),
    pub ope: Option<AddSub>,
    pub unarys: Vec<(Unary, Type)>,
}
impl Mul {
    pub fn is_lvar(&self) -> bool {
        if self.ope.is_some() || self.unarys.len() > 0 {
            return false;
        }
        self.first.0.is_lvar()
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
    pub unary: Box<(Unary, Type)>,
}
#[derive(Debug)]
pub struct UnaryVar {
    pub ope: Option<MulDiv>,
    pub prim: (Primary, Type),
    pub _arrs: Vec<Typed<Expr>>,
}
#[derive(Debug)]
pub enum Unary {
    Ptr(UnaryPtr),
    Var(UnaryVar),
}
impl Unary {
    pub fn is_lvar(&self) -> bool {
        match self {
            Unary::Var(p) => p.prim.0.is_lvar(),
            Unary::Ptr(p) => match p.ope {
                PtrOpe::Deref => false,
                PtrOpe::Ref => p.unary.0.is_lvar(),
            },
        }
    }
    pub fn ope(&self) -> &Option<MulDiv> {
        match self {
            Unary::Ptr(p) => p.unary.0.ope(),
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
    Num((String, Type)),
    Lv(Lvar),
    Expr(Box<Expr>),
    Fcall(Fcall),
}
#[derive(Debug)]
pub struct Primary {
    pub ope: Option<AddSub>,
    pub node: (PrimaryNode, Type),
}
#[derive(Debug)]
pub struct Ident {
    pub _type_: Type,
    pub offset: usize,
}
impl Primary {
    pub fn is_lvar(&self) -> bool {
        if self.ope.is_some() {
            return false;
        }
        match &self.node.0 {
            PrimaryNode::Lv(_) => true,
            _ => false,
        }
    }
}
