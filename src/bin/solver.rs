use std::io::Write;
use wordle::*;

fn main() {
    let db = load_database();
    let mut solver = Solver::new(&db);

    loop {
        let mut suggestions = solver.suggest(&db);

        let word: Word = loop {
            let suggestion = suggestions.pop();

            if let Some(suggestion) = suggestion {
                println!("Suggestion: {}", word_to_string(db[suggestion]));
            } else {
                eprintln!("No suggestions, sorry bud you're on your own!");
            }

            print!("Word (or enter, @): ");
            flush();

            let input = input::<String>().unwrap();

            if input == "@" {
                continue;
            }

            if let Some(suggestion) = suggestion {
                if input == "" {
                    break db[suggestion];
                }
            }

            if let Some(word) = str_to_word(&input) {
                break word;
            } else {
                println!("\"{}\" wasn't a valid word", input);
            }
        };

        let mut result = [LetterResult::NonMember; N_LETTERS];

        print!("Yellow: ");
        flush();
        let yellow = input::<String>().unwrap().to_uppercase();
        for c in yellow.chars() {
            for (idx, &letter) in word.iter().enumerate() {
                if letter == c {
                    result[idx] = LetterResult::Misplaced;
                }
            }
        }

        print!("Green: ");
        flush();
        let green = input::<String>().unwrap().to_uppercase();
        for c in green.chars() {
            for (idx, &letter) in word.iter().enumerate() {
                if letter == c {
                    result[idx] = LetterResult::Correct;
                }
            }
        }

        solver.inform(result, word);
    }
}

fn flush() {
    std::io::stdout().flush().unwrap();
}
