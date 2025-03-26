use super::node::Type;

pub const IDENTITY_OFFSET: usize = 8;
pub const RETURN: &str = "return";
pub const IF: &str = "if";
pub const WHILE: &str = "while";
pub const FOR: &str = "for";
pub const INT: &str = "int";
pub const TYPES: [&str; 1] = [INT];
pub const BLOCK_EXPECTED: &str = "block begin { expected";
pub const BRACE_NOT_BALANCED: &str = "brace{} not balanced";
pub const TYPE_WANTED: &str = "type declaration required";
pub const IDENTITY_WANTED: &str = "identity wanted";
pub fn sizeof(t: &Type) -> usize {
    match t {
        Type::_Panic => panic!("type Panic found"),
        Type::Int => 4, // 適切なレジスタを選択できていないので8固定
        Type::Ptr(_) => 8,
        Type::LInt => 4, // 数値で中身が不明ならIntとみなす
        Type::Array(_) => panic!(""),
    }
}
pub fn size_directive(t: &Type) -> String {
    match sizeof(t) {
        4 => "DWORD PTR ",
        8 | _ => "",
    }
    .into()
}
pub enum Register {
    _Ax,
    Di,
    Si,
    Dx,
    Cx,
    _8,
    _9,
}

pub fn register(size: usize, r: &Register) -> String {
    match size {
        4 => match r {
            Register::_Ax => "eax",
            Register::Di => "edi",
            Register::Si => "esi",
            Register::Dx => "edx",
            Register::Cx => "ecx",
            Register::_8 => "r8d",
            Register::_9 => "r9d",
        },
        8 | _ => match r {
            Register::_Ax => "rax",
            Register::Di => "rdi",
            Register::Si => "rsi",
            Register::Dx => "rdx",
            Register::Cx => "rcx",
            Register::_8 => "r8",
            Register::_9 => "r9",
        },
    }
    .into()
}
