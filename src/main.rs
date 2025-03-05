use std::env;
mod token;
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
    pop rax
    ret
        ",
        token::token::compile(raw_stmt).join("\n")
    );
    return;
}
