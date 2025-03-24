mod compiler;
fn main() {
    let tests = vec![
        "int main(){int x=10;int *y=&x;int **z=&y;**z=12;_p(x);int a=5;*z=&a;_p(**z);*y=2;_p(x);_p(a);_p(**z);_p(*&**&*&**&z);_p(*(y-8));return 0;}",
        // "int main(){int x=10;int *y=&x;int **z=&y;**z=12;_p(x);int a=5;*z=&a;_p(**z);*y=2;_p(x);_p(a);_p(**z);_p(*&**&*&**&z);_p(*(y-8));_p(***(y+8));return 0;}",
    ];
    // vec!["int main(){_p(  (( ( 3 + 4/2 ) * ( 2 + 2)) + 3) / ( ( ((2+3) *2) *2) + (4-1) )  );}"];
    for t in tests.iter() {
        print!("{}", compiler::compiler::compile((*t).into()).join("\n"));
    }
}
