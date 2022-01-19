use wordle::*;

fn main() {
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
