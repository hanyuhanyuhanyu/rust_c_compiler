use super::{generator::generate, parser::parse};
pub fn compile(input: String) -> Vec<String> {
    let parsed = parse(&input);
    if parsed.is_err() {
        let err_msg = parsed.unwrap_err();
        panic!(
            "{}\nat line {}\n    {}\n{}^ something wrong here",
            err_msg.reason,
            err_msg.read_line + 1,
            err_msg.source.unwrap_or("no original source given".into()),
            "    ".to_string() + &" ".repeat(err_msg.index)
        )
    }
    match generate(&parsed.unwrap()) {
        Err(e) => panic!("{}", e.join("\n")),
        Ok(e) => return e,
    }
}
