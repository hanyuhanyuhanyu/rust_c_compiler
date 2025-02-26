use std::env;
fn build_calc_lines(nums: Vec<i32>, opes: Vec<char>, space_count: usize) -> String {
    // TODO細かいチェックはぬき
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!("mov rax, {}", nums[0]).into());
    for (i, ope) in opes.iter().enumerate() {
        let instruction: String = match ope {
            '+' => "add",
            '-' => "sub",
            _ => "",
        }
        .into();
        lines.push(format!("{} rax, {}", instruction, nums[i + 1]).into());
    }
    return lines
        .iter()
        .map(|l| format!("{}{}", " ".repeat(space_count), l))
        .collect::<Vec<String>>()
        .join("\n");
}
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("no arg given");
        return;
    }
    let mut nums: Vec<i32> = Vec::new();
    let mut opes: Vec<char> = Vec::new();
    let mut buffer: i32 = 0;
    for c in args[1].chars() {
        if c.is_numeric() {
            buffer *= 10;
            buffer += c.to_digit(10).unwrap() as i32;
            continue;
        }
        match c {
            '+' | '-' => {
                nums.push(buffer);
                opes.push(c);
            }
            _ => {
                println!("unexpected char: {}", c);
                return;
            }
        }
        buffer = 0;
    }
    nums.push(buffer);
    println!(
        ".intel_syntax noprefix
.globl main
main:
{}
    ret
        ",
        build_calc_lines(nums, opes, 4)
    );
    return;
}
