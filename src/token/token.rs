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
                None => break,
                _ => break,
            }
        }
    }
    fn num(&mut self, initial: char) -> ConsumeResult {
        let mut number: String = initial.into();
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
                    return self.num(c);
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
    fn mul(&mut self) -> ConsumeResult {
        let first = self.primary();
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
            let next = self.primary();
            if next.is_some() {
                return next;
            }
            self.push("pop rdi".into());
            self.push("pop rax".into());
            if ope == "mul" {
                self.push("imul rax, rdi".into());
            } else if ope == "div" {
                self.push("cqu".into());
                self.push("idiv rax, rdi".into());
            }
            self.push("push rax".into())
        }
        None
    }
    pub fn expr(&mut self) -> ConsumeResult {
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
