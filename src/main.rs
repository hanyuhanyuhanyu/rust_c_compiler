use std::env;
mod compiler;
fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("no arg given");
        return;
    }
    let raw_stmt = args[1].clone();

    println!(
        ".intel_syntax noprefix
.globl main
main:
{}
",
        compiler::compiler::compile(raw_stmt)
            .iter()
            .map(|a| format!("    {}", a))
            .collect::<Vec<String>>()
            .join("\n")
    );
    return;
}
