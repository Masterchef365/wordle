use std::collections::{HashSet, HashMap};
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;

const N_LETTERS: usize = 5;
const MAX_ATTEMPTS: usize = 6;

type Word = [char; N_LETTERS];

fn letter_idx(c: char) -> usize {
    c as usize - 'A' as usize
}

fn calc_optimal_first_word(dict: &[Word]) -> Word {
    let mut letter_hist = [0u64; 26];

    for word in dict {
        for letter in word {
            if ('A'..='Z').contains(letter) {
                letter_hist[letter_idx(*letter)] += 1;
            } else {
                panic!("Wrong letter {}", letter);
            }
        }
    }

    let mut best_word = dict[0];
    let mut best_score = 0;

    for word in dict {
        let mut letter_used = [false; 26];
        let mut repeats = 1;
        let mut total = 0;
        for letter in word {
            let idx = letter_idx(*letter);
            if letter_used[idx] {
                repeats += 1;
            }

            total += letter_hist[idx];

            letter_used[idx] = true;
        }

        let score = total / repeats;

        if score > best_score {
            best_score = score;
            best_word = *word;
        }
    }

    best_word
}

fn main() {
    let database = load_database();
    dbg!(calc_optimal_first_word(&database));
}

fn pmain() {
    let database = load_database();
    //dbg!(play_against_self(&database, str_to_word("panic").unwrap()));

    let mut n_wins = 0;
    let mut n_losses = 0;
    let mut n_exhausts = 0;
    
    for (idx, &word) in database.iter().enumerate() {
        let result = play_against_self(&database, word);
        match result {
            Some(GameResult::Loss) => n_losses += 1,
            Some(GameResult::Win) => n_wins += 1,
            None => n_exhausts += 1,
            _ => (),
        }

        if idx % 1000 == 0 {
            dbg!(idx, n_wins, n_losses, n_exhausts);
        }
    }
    dbg!(n_wins, n_losses, n_exhausts);
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

#[derive(Copy, Clone, PartialEq, Debug)]
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
        assert_eq!(
            attempt(&mut game, "panic"),
            GameResult::Win,
        );

        assert_eq!(
            attempt(&mut game_fork, "binks"),
            GameResult::Loss,
        );
    }
}

struct Solver {
    non_members: HashSet<char>,
    misplaced: HashMap<char, HashSet<usize>>,
    correct: [Option<char>; N_LETTERS],
}

impl Solver {
    fn new() -> Self {
        Self {
            non_members: HashSet::new(),
            misplaced: HashMap::new(),
            correct: [None; N_LETTERS],
        }
    }

    fn suggest(&self, dictionary: &[Word]) -> Vec<usize> {
        let mut suggestions = vec![];
        'words: for (idx, word) in dictionary.iter().enumerate() {
            for letter in self.misplaced.keys() {
                if !word.contains(letter) {
                    continue 'words;
                }
            }

            for (idx, letter) in word.iter().enumerate() {
                if let Some(positions) = self.misplaced.get(letter) {
                    if positions.contains(&idx) {
                        continue 'words;
                    }
                }
            }

            for letter in &self.non_members {
                if word.contains(&letter) {
                    continue 'words;
                }
            }

            for (c, w) in self.correct.iter().zip(word) {
                if let Some(c) = c {
                    if w != c {
                        continue 'words;
                    }
                }
            }

            suggestions.push(idx);
        }

        suggestions 
    }

    fn inform(&mut self, result: [LetterResult; N_LETTERS], word: Word) {
        //dbg!(&self.must_avoid, &self.must_contain, &self.correct);
        for (i, r) in result.iter().enumerate() {
            let c = word[i];
            match r {
                LetterResult::Correct => {
                    self.misplaced.entry(c).or_default();
                    self.correct[i] = Some(c);
                },
                LetterResult::Misplaced => {
                    self.misplaced.entry(c).or_default().insert(i);
                },
                LetterResult::NonMember => {
                    self.non_members.insert(c);
                }
            }
        }
    }
}

fn play_against_self(dictionary: &[Word], word: Word) -> Option<GameResult> {
    let mut game = Game::new(word);
    let mut solver = Solver::new();
    let mut play = str_to_word("acorn").unwrap(); // TODO: Replace with word containing most common chars from the dictionary!
    loop {
        //dbg!(play);
        let result = game.attempt(play);
        //dbg!(result);
        match result {
            GameResult::Miss(result) => solver.inform(result, play),
            other => break Some(other),
        }
        let suggestions = solver.suggest(dictionary);
        play = suggestions.get(0).map(|&i| dictionary[i])?;
    }
}
