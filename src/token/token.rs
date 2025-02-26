#[derive(Debug, PartialEq)]
enum TokenType {
    Number,
    Operator,
    Unknown,
}
enum OperateType {
    Add,
    Sub,
    Unknown,
}
trait Tokenizable {
    fn token_type(&self) -> TokenType;
}
impl Tokenizable for char {
    fn token_type(&self) -> TokenType {
        if self.is_numeric() {
            return TokenType::Number;
        }
        match self {
            '+' | '-' => TokenType::Operator,
            _ => TokenType::Unknown,
        }
    }
}
#[derive(Debug)]
pub(crate) struct Token {
    raw: String,
    expect: TokenType,
    kind: TokenType,
}
impl Token {
    const fn new() -> Token {
        Token {
            raw: String::new(),
            expect: TokenType::Unknown,
            kind: TokenType::Unknown,
        }
    }
    fn consume(&mut self, c: char) -> bool {
        if !self.consumable(c) {
            return false;
        }
        if c.token_type() == TokenType::Unknown {
            return true;
        }
        if self.empty() {
            self.raw = c.into();
            self.expect = c.token_type();
            self.kind = c.token_type();
            return true;
        }
        self.raw.push(c);
        return true;
    }
    fn empty(&self) -> bool {
        return self.raw.len() == 0;
    }
    fn consumable(&self, c: char) -> bool {
        return self.empty() || self.kind == c.token_type();
    }
    fn is_num(&self) -> bool {
        return self.kind == TokenType::Number;
    }
    fn to_string(&self) -> String {
        return self.raw.clone();
    }
    fn ope(&self) -> OperateType {
        match self.raw.chars().next().unwrap() {
            '+' => OperateType::Add,
            '-' => OperateType::Sub,
            _ => OperateType::Unknown,
        }
    }
}

fn parse(source: String) -> Vec<Token> {
    if source.len() == 0 {}
    let mut tokens = Vec::new();
    let mut current_token = Token::new();
    for c in source.chars() {
        let consumed = current_token.consume(c);
        if consumed {
            continue;
        }
        tokens.push(current_token);
        current_token = Token::new();
        current_token.consume(c);
    }
    tokens.push(current_token);
    return tokens;
}

fn compile(tokens: Vec<Token>, space_count: usize) -> Vec<String> {
    let mut lines = Vec::new();
    if tokens.len() == 0 {
        return lines;
    }
    let first = tokens.first().unwrap();
    if !first.is_num() {
        panic!("first token is not number");
    }
    let space = " ".repeat(space_count);
    let mut ind = 1;
    lines.push(format!("{}mov rax, {}", space, first.to_string()).into());

    while ind < tokens.len() {
        let current = tokens.get(ind).unwrap();
        if current.is_num() {
            panic!("unexpected number");
        }
        let next = tokens.get(ind + 1).unwrap();
        if !next.is_num() {
            panic!("number must comes next to operator");
        }
        let instruction = match current.ope() {
            OperateType::Add => "add",
            OperateType::Sub => "sub",
            OperateType::Unknown => panic!("unknown operator"),
        };
        lines.push(format!(
            "{}{} rax, {}",
            space,
            instruction,
            next.to_string()
        ));
        ind += 2;
    }
    return lines;
}

pub fn compile_from_raw(str: String, space_count: usize) -> Vec<String> {
    return compile(parse(str), space_count);
}
