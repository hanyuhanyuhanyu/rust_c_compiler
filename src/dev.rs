mod token;
fn main() {
    token::token::compile(" ( (3 / (4-3)) * (  5+ 6 * 7 ) ) ".into());
}
