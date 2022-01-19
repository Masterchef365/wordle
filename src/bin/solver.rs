use wordle::*;

fn input() -> String {
    let mut output = "".into();
    std::io::stdin().read_line(&mut output).expect("IO Error");
    output
}

fn main() {
    dbg!(input());
}
