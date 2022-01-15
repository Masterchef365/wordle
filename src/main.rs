use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;

const N_LETTERS: usize = 5;
const MAX_ATTEMPTS: usize = 6;

type Word = [char; N_LETTERS];

fn main() {
    let database = load_database();
    //dbg!(database.len());
    dbg!(database);
}

fn load_database() -> Vec<Word> {
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

    read_to_string(database_path)
        .expect("Failed to load database")
        .lines()
        .filter_map(str_to_word)
        .collect()
}

struct Game {
    word: Word,
    attempts: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum LetterResult {
    Correct,
    Misplaced,
    NonMember,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum GameResult {
    Win,
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
            GameResult::Win
        } else {
            if self.attempts == MAX_ATTEMPTS {
                GameResult::Loss
            } else {
                GameResult::Miss(result)
            }
        }
    }
}

fn input<T: FromStr>() -> Option<T> {
    BufReader::new(std::io::stdin())
        .lines()
        .next()
        .transpose()
        .ok()?
        .and_then(|line| line.trim().parse().ok())
}

fn str_to_word(s: &str) -> Option<Word> {
    let mut word = [char::REPLACEMENT_CHARACTER; N_LETTERS];
    let s = s.to_uppercase(); // TODO: Don't allocate!
    if s.chars().count() != N_LETTERS {
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
        let mut attempt = |s: &str| game.attempt(str_to_word(s).unwrap());
        assert_eq!(
            attempt("fovea"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::NonMember,
                Lr::NonMember,
                Lr::NonMember,
                Lr::Misplaced,
            ])
        );
        assert_eq!(
            attempt("grads"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::NonMember,
                Lr::Misplaced,
                Lr::NonMember,
                Lr::NonMember,
            ])
        );
        assert_eq!(
            attempt("quack"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::NonMember,
                Lr::Misplaced,
                Lr::Misplaced,
                Lr::NonMember,
            ])
        );
        assert_eq!(
            attempt("tacit"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::Correct,
                Lr::Misplaced,
                Lr::Correct,
                Lr::NonMember,
            ])
        );
        assert_eq!(
            attempt("manic"),
            GameResult::Miss([
                Lr::NonMember,
                Lr::Correct,
                Lr::Correct,
                Lr::Correct,
                Lr::Correct,
            ])
        );
        assert_eq!(
            attempt("panic"),
            GameResult::Win,
        );
    }
}