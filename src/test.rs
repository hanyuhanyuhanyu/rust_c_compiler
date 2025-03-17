mod compiler;
fn main() {
    let tests = vec!["a = 0; b=1; if (a < 1) { b = b +2; a = a + 5; } return a+b;"];
    for t in tests.iter() {
        print!("{:?}", compiler::compiler::compile((*t).into()));
    }
}
