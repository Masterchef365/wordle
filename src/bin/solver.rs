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

        println!("Word:  {}", word_to_string(word));
        print!("Reply: ");
        flush();

        let mut result = [LetterResult::NonMember; N_LETTERS];
        let input = input::<String>().unwrap();
        for (r, c) in result.iter_mut().zip(input.chars()) {
            *r = match c {
                'y' | 'Y' => LetterResult::Misplaced,
                'g' | 'G' => LetterResult::Correct,
                _ => LetterResult::NonMember,
            }
        }

        dbg!(result);

        solver.inform(result, word);
    }
}

fn flush() {
    std::io::stdout().flush().unwrap();
}
