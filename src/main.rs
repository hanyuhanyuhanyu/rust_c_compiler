use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("no arg given");
        return;
    }
    println!(".intel_syntax noprefix
.globl main
main:
        mov rax,{} 
        ret
    ", args[1]);

}
