use std::collections::HashMap;

use super::{generator::generate, parser::Parser};
pub fn compile(input: String) -> Vec<String> {
    let parsed = Parser {
        index: 0,
        input: &input,
        ident_count: 0,
        idents: HashMap::new(),
    }
    .parse();
    if parsed.is_err() {
        let err_msg = parsed.unwrap_err();
        panic!(
            "{}\n{}\n{}^",
            err_msg.reason,
            input,
            " ".repeat(err_msg.index)
        )
    }
    match generate(&parsed.unwrap()) {
        Err(e) => panic!("{}", e.join("\n")),
        Ok(e) => return e,
    }
}
