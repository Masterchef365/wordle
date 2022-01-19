use wordle::*;

fn main() {
    let database = load_database();
    dbg!(calc_optimal_first_word(&database));
}

fn letter_idx(c: char) -> usize {
    c as usize - 'A' as usize
}

fn calc_optimal_first_word(dict: &[Word]) -> Word {
    let mut letter_hists = [[0u64; 26]; 5];

    for word in dict {
        for (slot, letter) in word.iter().enumerate() {
            if ('A'..='Z').contains(letter) {
                letter_hists[slot][letter_idx(*letter)] += 1;
            } else {
                panic!("Wrong letter {}", letter);
            }
        }
    }

    /*
    let mut k: Vec<(usize, u64)> = letter_hist.iter().copied().enumerate().collect();
    k.sort_by_key(|(_, count)| *count);
    for (idx, count) in k {
        dbg!((char::from('A' as u8 + idx as u8), count));
    }
    */

    let mut best_word = dict[0];
    let mut best_score = 0;

    for word in dict {
        let mut letter_used = [false; 26];
        let mut repeats = 1;
        let mut total = 0;
        for (slot, letter) in word.iter().enumerate() {
            let idx = letter_idx(*letter);
            if letter_used[idx] {
                repeats += 1;
            }

            total += letter_hists[slot][idx];

            letter_used[idx] = true;
        }

        let score = total / repeats;

        if score > best_score {
            best_score = score;
            best_word = *word;
            //dbg!(best_score, word.iter().copied().collect::<String>());
        }
    }


    best_word
}
