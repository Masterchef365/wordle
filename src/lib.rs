use std::collections::HashSet;
use std::fs::read_to_string;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub const N_LETTERS: usize = 5;
pub const MAX_ATTEMPTS: usize = 6;

pub type Word = [char; N_LETTERS];

pub fn load_database() -> Vec<Word> {
    let database_path = Path::new("database.txt");

    let database_path: PathBuf = if database_path.is_file() {
        database_path.into()
    } else {
        if let Some(path) = std::env::args().skip(1).next() {
            eprintln!(
                "{} not found, loading wordlist from {}",
                database_path.display(),
                path
            );
            path.into()
        } else {
            panic!(
                "{} not found, please specify a word list to import on the command line!",
                database_path.display()
            );
        }
    };

    let mut db: Vec<Word> = read_to_string(database_path)
        .expect("Failed to load database")
        .lines()
        .filter_map(str_to_word)
        .collect();

    use rand::prelude::*;
    db.shuffle(&mut rand::thread_rng());

    db
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Game {
    word: Word,
    attempts: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum LetterResult {
    Correct,
    Misplaced,
    NonMember,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum GameResult {
    Win(usize),
    Miss([LetterResult; N_LETTERS]),
    Loss,
}

impl Game {
    pub fn new(word: Word) -> Self {
        Self { word, attempts: 0 }
    }

    pub fn attempt(&mut self, word: Word) -> GameResult {
        let mut result = [LetterResult::NonMember; N_LETTERS];
        for ((goal, attempt), result) in self.word.iter().zip(&word).zip(&mut result) {
            if attempt == goal {
                *result = LetterResult::Correct;
            } else {
                if self.word.contains(attempt) {
                    *result = LetterResult::Misplaced;
                }
            }
        }

        self.attempts += 1;

        if result == [LetterResult::Correct; N_LETTERS] {
            GameResult::Win(self.attempts)
        } else {
            if self.attempts == MAX_ATTEMPTS {
                GameResult::Loss
            } else {
                GameResult::Miss(result)
            }
        }
    }
}

pub fn input<T: FromStr>() -> Option<T> {
    BufReader::new(std::io::stdin())
        .lines()
        .next()
        .transpose()
        .ok()?
        .and_then(|line| line.trim_end_matches('\n').parse().ok())
}

pub fn str_to_word(s: &str) -> Option<Word> {
    let mut word = [char::REPLACEMENT_CHARACTER; N_LETTERS];
    let s = s.to_uppercase(); // TODO: Don't allocate!

    if s.chars().count() != N_LETTERS {
        return None;
    }

    if s.chars().any(|c| !('A'..='Z').contains(&c)) {
        return None;
    }

    word.iter_mut().zip(s.chars()).for_each(|(w, s)| *w = s);

    Some(word)
}

#[cfg(test)]
mod tests {
    use super::LetterResult as Lr;
    use super::*;

    #[test]
    fn test_game() {
        let word = str_to_word("panic").unwrap();
        let mut game = Game::new(word);
        let mut attempt = |game: &mut Game, s: &str| game.attempt(str_to_word(s).unwrap());
        assert_eq!(
            attempt(&mut game, "fovea"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::NonMember,
                Lr::NonMember,
                Lr::NonMember,
                Lr::Misplaced,
            ])
        );
        assert_eq!(
            attempt(&mut game, "grads"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::NonMember,
                Lr::Misplaced,
                Lr::NonMember,
                Lr::NonMember,
            ])
        );
        assert_eq!(
            attempt(&mut game, "quack"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::NonMember,
                Lr::Misplaced,
                Lr::Misplaced,
                Lr::NonMember,
            ])
        );
        assert_eq!(
            attempt(&mut game, "tacit"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::Correct,
                Lr::Misplaced,
                Lr::Correct,
                Lr::NonMember,
            ])
        );
        assert_eq!(
            attempt(&mut game, "manic"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::Correct,
                Lr::Correct,
                Lr::Correct,
                Lr::Correct,
            ])
        );
        let mut game_fork = game.clone();
        assert!(
            matches!(attempt(&mut game, "panic"),
            GameResult::Win(_)),
        );

        assert_eq!(
            attempt(&mut game_fork, "binks"),
            GameResult::Loss,
        );
    }
}

// TODO: In ties, prefer words which have letters that are more common! (Like the best starting word solver)
pub struct Solver {
    /// Words must have these letters
    must_have: HashSet<char>,
    /// Letters in the given position cannot match these
    misplaced: [[bool; 26]; N_LETTERS],
    /// Words must have these letters in their respective positions
    correct: [Option<char>; N_LETTERS],
    /// Letter histograms representative of the dictionary
    pub letter_hists: LetterHists,
}

impl Solver {
    pub fn new(dictionary: &[Word]) -> Self {
        Self {
            must_have: HashSet::new(),
            misplaced: [[false; 26]; N_LETTERS],
            correct: [None; N_LETTERS],
            letter_hists: calc_letter_hist(dictionary),
        }
    }

    pub fn suggest(&self, dictionary: &[Word]) -> Vec<usize> {
        let mut suggestions = vec![];
        'words: for (idx, word) in dictionary.iter().enumerate() {

            for ((&letter, misplaced), &correct) in word.iter().zip(&self.misplaced).zip(&self.correct) {
                if let Some(correct) = correct {
                    if letter != correct {
                        continue 'words;
                    }
                } else {
                    if misplaced[letter_idx(letter)] {
                        continue 'words;
                    }
                }
            }

            // Make sure the words has all of the letters it must have
            for must in &self.must_have {
                if !word.contains(must) {
                    continue 'words;
                }
            }

            suggestions.push(idx);
        }

        // Sort by score
        suggestions.sort_by_key(|&word| score_word(dictionary[word], &self.letter_hists));

        suggestions 
    }

    pub fn inform(&mut self, result: [LetterResult; N_LETTERS], word: Word) {
        //dbg!(&self.must_avoid, &self.must_contain, &self.correct);
        for (idx, r) in result.iter().enumerate() {
            let c = word[idx];
            match r {
                LetterResult::Correct => {
                    self.must_have.insert(c);
                    self.correct[idx] = Some(c);
                },
                LetterResult::Misplaced => {
                    self.must_have.insert(c);
                    self.misplaced[idx][letter_idx(c)] = true;
                },
                LetterResult::NonMember => {
                    for misplaced in &mut self.misplaced {
                        if self.correct[idx].is_none() {
                            misplaced[letter_idx(c)] = true;
                        }
                    }
                }
            }
        }
    }
}

pub fn play_against_self(dictionary: &[Word], word: Word) -> Option<GameResult> {
    //eprintln!();
    //eprintln!("##################################################");
    //eprintln!();

    let mut game = Game::new(word);
    let mut solver = Solver::new(dictionary);
    loop {
        let suggestions = solver.suggest(dictionary);
        let play = suggestions.last().map(|&i| dictionary[i])?;
        //dbg!(play);

        let result = game.attempt(play);
        //dbg!(result);

        match result {
            GameResult::Miss(result) => solver.inform(result, play),
            other => break Some(other),
        }
    }
}

type LetterHists = [[u64; 26]; N_LETTERS];

pub fn score_word(word: Word, letter_hists: &LetterHists) -> u64 {
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

    total / repeats
}

fn calc_letter_hist(dict: &[Word]) -> LetterHists {
    let mut letter_hists = [[0u64; 26]; N_LETTERS];

    for word in dict {
        for (slot, letter) in word.iter().enumerate() {
            if ('A'..='Z').contains(letter) {
                letter_hists[slot][letter_idx(*letter)] += 1;
            } else {
                panic!("Wrong letter {}", letter);
            }
        }
    }

    letter_hists
}

fn letter_idx(c: char) -> usize {
    c as usize - 'A' as usize
}

pub fn word_to_string(word: Word) -> String {
    word.iter().collect::<String>()
}
