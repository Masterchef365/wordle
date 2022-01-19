use wordle::*;

fn main() {
    let db = load_database();
    let mut solver = Solver::new(&db);

    let tests = [
        "stern",
        "bares",
        "jazzy",
        "fruity",
    ];

    for test in tests {
        if let Some(score) = str_to_word(test).map(|t| score_word(t, &solver.letter_hists)) {
            println!("{}: {}", test, score);
        } else {
            println!("{}: No score!", test);
        }
    }

    panic!();
    loop {
        let suggestion = match solver.suggest(&db).last() {
            Some(s) => *s,
            None => {
                eprintln!("No suggestions, sorry bud you're on your own!");
                break;
            }
        };

        println!("Suggestion: {}", word_to_string(db[suggestion]));

        println!("Yellow: ");
        //let yellow_indices = input::<String>().
    }
}
