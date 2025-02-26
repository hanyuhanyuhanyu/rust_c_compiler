mod token;
fn main() {
    println!(
        "{:?}",
        token::token::compile_from_raw("6 -1  + 3".into(), 4)
    );
}
