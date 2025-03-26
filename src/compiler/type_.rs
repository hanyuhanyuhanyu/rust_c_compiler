#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    _Panic, // 開発用
    LInt,
    Int,
    Ptr(Box<Type>),
    Array(Box<Type>), // TODO sizeいる
}
impl Type {
    pub fn when_addsub(&self, register: String) -> Vec<String> {
        match &self {
            Type::LInt | Type::Int => vec![],
            Type::Ptr(_) => vec![format!("imul {}, {}", register, self.sizeof())],
            _ => vec![],
        }
    }
    pub fn sizeof(&self) -> usize {
        match self {
            Type::_Panic => panic!("type Panic found"),
            Type::Int => 4, // 適切なレジスタを選択できていないので8固定
            Type::Ptr(_) => 8,
            Type::LInt => 4, // 数値で中身が不明ならIntとみなす
            Type::Array(_) => panic!(""),
        }
    }
    pub fn can_be_for_array_index(&self) -> bool {
        match self {
            Type::Int | Type::LInt => true,
            _ => false,
        }
    }
}
