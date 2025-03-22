use super::node::Type;

pub const IDENTITY_OFFSET: usize = 8;
pub const RETURN: &str = "return";
pub const IF: &str = "if";
pub const WHILE: &str = "while";
pub const FOR: &str = "for";
pub const INT: &str = "int";
pub const TYPES: [&str; 1] = [INT];
pub const BLOCK_EXPECTED: &str = "block begin { expected";
pub const BRACKET_NOT_BALANCED: &str = "bracket{} not balanced";
pub const TYPE_WANTED: &str = "type declaration required";
pub const IDENTITY_WANTED: &str = "identity wanted";
pub fn sizeof(t: &Type) -> usize {
    match t {
        Type::Int => 8, // 適切なレジスタを選択できていないので8固定
        Type::Ptr(_) => 8,
    }
}
