use super::type_::Type;

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
pub const LEFT_VALUE_IS_NOT_ASSIGNABLE: &str = "left value is not assignable";
pub const NOT_AVAILABLE_FOR_ARRAY_INDEX: &str = "this type is not available for array index";
pub fn size_directive(t: &Type) -> String {
    match t.sizeof() {
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
