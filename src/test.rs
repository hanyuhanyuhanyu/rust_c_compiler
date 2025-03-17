mod compiler;
fn main() {
    let tests = vec!["int main() {return 1;}"];
    for t in tests.iter() {
        print!("{}", compiler::compiler::compile((*t).into()).join("\n"));
    }
}
