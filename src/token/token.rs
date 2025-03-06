#[derive(Debug)]
struct ConsumeFailure {
    index: usize,
    reason: String,
}
type ConsumeResult = Option<ConsumeFailure>;

const UNEXPECTED_FAIL: &str = "unexpected failure";
struct Consuming {
    index: usize,
    input: String,
    queue: Vec<String>,
    tab_index: usize,
}
impl Consuming {
    fn eat_one(&mut self) -> Option<char> {
        self.space();
        let char = self.input.chars().nth(self.index);
        self.index += 1;
        char
    }
    fn wind(&mut self) {
        self.index -= 1;
    }
    fn seek(&mut self, checker: fn(check: char) -> bool) -> Option<char> {
        self.space();
        match self.input.chars().nth(self.index) {
            None => None,
            Some(c) => {
                if checker(c) {
                    self.index += 1;
                    Some(c)
                } else {
                    None
                }
            }
        }
    }
    fn try_eat(&mut self, str: &str) -> bool {
        self.space();
        let max = self.index + str.len();
        if self.input.len() <= max {
            return false;
        }
        if &self.input[self.index..max] == str {
            self.index += str.len();
            return true;
        }
        false
    }
    fn push(&mut self, str: String) {
        self.queue
            .push(format!("{}{}", " ".repeat(self.tab_index), str));
    }
    fn space(&mut self) {
        loop {
            match self.input.chars().nth(self.index) {
                Some(' ') => {
                    self.index += 1;
                }
                _ => break,
            }
        }
    }
    fn num(&mut self) -> ConsumeResult {
        let mut number: String = "".into();
        loop {
            match self.seek(|c: char| -> bool { c.is_numeric() }) {
                None => break,
                Some(c) => {
                    if c.is_numeric() {
                        number.push(c.into());
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        if number.is_empty() {
            Some(ConsumeFailure {
                index: self.index,
                reason: "number expected".into(),
            })
        } else {
            self.push(format!("push {}", number));
            None
        }
    }
    fn primary(&mut self) -> ConsumeResult {
        match self.eat_one() {
            None => {
                return Some(ConsumeFailure {
                    index: self.index,
                    reason: UNEXPECTED_FAIL.into(),
                });
            }
            Some(c) => {
                if c.is_numeric() {
                    self.wind();
                    return self.num();
                }
                if c != '(' {
                    return Some(ConsumeFailure {
                        index: self.index,
                        reason: "only number or ( acceptable".into(),
                    });
                }
            }
        }
        let res = self.expr();
        if res.is_some() {
            return res;
        }
        match self.eat_one() {
            None => {
                return Some(ConsumeFailure {
                    index: self.index,
                    reason: UNEXPECTED_FAIL.into(),
                });
            }
            Some(')') => None,
            _ => Some(ConsumeFailure {
                index: self.index,
                reason: "parenthesis unbalanced".into(),
            }),
        }
    }
    fn unary(&mut self) -> ConsumeResult {
        match self.eat_one() {
            None => {
                return Some(ConsumeFailure {
                    index: self.index,
                    reason: UNEXPECTED_FAIL.into(),
                });
            }
            Some('+') => {
                return self.primary();
            }
            Some('-') => {
                self.push(format!("push {}", 0));
                let res = self.primary();
                if res.is_some() {
                    return res;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("sub rax, rdi".into());
                self.push("push rax".into());
                None
            }
            _ => {
                self.wind();
                return self.primary();
            }
        }
    }
    fn mul(&mut self) -> ConsumeResult {
        let first = self.unary();
        if first.is_some() {
            return first;
        }
        loop {
            let char = self.seek(|c| -> bool { c == '*' || c == '/' });
            if char.is_none() {
                break;
            }
            let ope = match char.unwrap() {
                '*' => "mul",
                '/' => "div",
                _ => "",
            };
            if ope.is_empty() {
                break;
            }
            let next = self.unary();
            if next.is_some() {
                return next;
            }
            self.push("pop rdi".into());
            self.push("pop rax".into());
            if ope == "mul" {
                self.push("imul rax, rdi".into());
            } else if ope == "div" {
                self.push("cqo".into());
                self.push("idiv rax, rdi".into());
            }
            self.push("push rax".into())
        }
        None
    }
    fn add(&mut self) -> ConsumeResult {
        let first = self.mul();
        if first.is_some() {
            return first;
        }
        loop {
            let char = self.seek(|c| -> bool { c == '+' || c == '-' });
            if char.is_none() {
                break;
            }
            let ope = match char.unwrap() {
                '+' => "add",
                '-' => "sub",
                _ => "",
            };
            if ope.is_empty() {
                break;
            }
            let next = self.mul();
            if next.is_some() {
                return next;
            }
            self.push("pop rdi".into());
            self.push("pop rax".into());
            if ope == "add" {
                self.push("add rax, rdi".into());
            } else if ope == "sub" {
                self.push("sub rax, rdi".into());
            }
            self.push("push rax".into());
        }
        return first;
    }
    fn relational(&mut self) -> ConsumeResult {
        let first = self.add();
        if first.is_some() {
            return first;
        }
        loop {
            if self.try_eat("<=") {
                let later = self.add();
                if later.is_some() {
                    return later;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("cmp rax, rdi".into());
                self.push("setle al".into());
                self.push("movzb rax, al".into());
                self.push("push rax".into());
            } else if self.try_eat(">=") {
                let later = self.add();
                if later.is_some() {
                    return later;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("cmp rdi, rax".into());
                self.push("setle al".into());
                self.push("movzb rax, al".into());
                self.push("push rax".into());
            } else if self.try_eat("<") {
                let later = self.add();
                if later.is_some() {
                    return later;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("cmp rax, rdi".into());
                self.push("setl al".into());
                self.push("movzb rax, al".into());
                self.push("push rax".into());
            } else if self.try_eat(">") {
                let later = self.add();
                if later.is_some() {
                    return later;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("cmp rdi, rax".into());
                self.push("setl al".into());
                self.push("movzb rax, al".into());
                self.push("push rax".into());
            } else {
                break;
            }
        }
        None
    }
    fn equality(&mut self) -> ConsumeResult {
        let first = self.relational();
        if first.is_some() {
            return first;
        }
        loop {
            if self.try_eat("==") {
                let later = self.relational();
                if later.is_some() {
                    return later;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("cmp rax, rdi".into());
                self.push("sete al".into());
                self.push("movzb rax, al".into());
                self.push("push rax".into());
            } else if self.try_eat("!=") {
                let later = self.relational();
                if later.is_some() {
                    return later;
                }
                self.push("pop rdi".into());
                self.push("pop rax".into());
                self.push("cmp rax, rdi".into());
                self.push("setne al".into());
                self.push("movzb rax, al".into());
                self.push("push rax".into());
            } else {
                break;
            }
        }
        None
    }
    pub fn expr(&mut self) -> ConsumeResult {
        self.equality()
    }
}
pub fn compile(input: String) -> Vec<String> {
    let mut consumer = Consuming {
        index: 0,
        input: input,
        queue: Vec::new(),
        tab_index: 4,
    };

    match consumer.expr() {
        None => consumer.queue,
        Some(ConsumeFailure { index, reason }) => {
            let mut v = Vec::new();
            v.push(reason);
            v.push(consumer.input);
            v.push(format!("{}{}", " ".repeat(index), "^"));
            v
        }
    }
}
