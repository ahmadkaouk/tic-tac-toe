use cosmwasm_schema::cw_serde;
use thiserror::Error;

/// A player in the game.
#[cw_serde]
#[derive(Copy)]
pub enum Player {
    X,
    O,
    None,
}

/// An error that can occur when playing a game.
#[derive(Error, Debug, PartialEq)]
pub enum GameError {
    /// The player tried to play out of turn.
    #[error("Not your turn")]
    NotYourTurn,
    /// The player tried to play on an occupied or an invalid cell.
    #[error("Cell {0} is already occupied")]
    InvalidMove(usize),
}

/// The winning combinations of tic-tac-toe.
const WINNING_COMBINATIONS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

/// A tic-tac-toe game.
#[cw_serde]
#[derive(Copy)]
pub struct Game {
    board: [Player; 9],
    turn: Player,
}

impl Game {
    /// Creates a new game with with an empty board and `X` as the first player.
    pub fn new() -> Game {
        Game {
            board: [Player::None; 9],
            turn: Player::X,
        }
    }

    /// Plays a move on the board.
    pub fn play(&mut self, player: Player, index: usize) -> Result<(), GameError> {
        if self.turn != player {
            return Err(GameError::NotYourTurn);
        }

        // Check if the index is valid and the cell is empty.
        let cell = match self.board.get_mut(index) {
            Some(cell) if *cell == Player::None => cell,
            _ => return Err(GameError::InvalidMove(index)),
        };

        *cell = player;

        // Switch turns.
        self.turn = match player {
            Player::X => Player::O,
            Player::O => Player::X,
            Player::None => Player::None,
        };
        Ok(())
    }

    /// Get the winner of the game. Returns `None` if there is no winner yet.
    pub fn winner(&self) -> Option<Player> {
        for combination in &WINNING_COMBINATIONS {
            let player = self.board[combination[0]];
            if player != Player::None && combination.iter().all(|&i| self.board[i] == player) {
                return Some(player);
            }
        }
        None
    }

    /// Checks if the game is over. A game is over if there is a winner or if the board is full.
    pub fn is_over(&self) -> bool {
        self.winner().is_some() || self.board.iter().all(|&p| p != Player::None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_game() {
        let game = Game::new();

        assert_eq!(game.board, [Player::None; 9]);
        assert_eq!(game.turn, Player::X);
    }

    #[test]
    fn test_invalid_move() {
        let mut game = Game::new();

        assert_eq!(game.play(Player::X, 0), Ok(()));
        assert_eq!(game.play(Player::O, 0), Err(GameError::InvalidMove(0)));
        assert_eq!(game.play(Player::O, 10), Err(GameError::InvalidMove(10)));
        assert_eq!(game.play(Player::O, 8), Ok(()));
    }

    #[test]
    fn test_not_your_turn() {
        let mut game = Game::new();

        assert_eq!(game.play(Player::O, 0), Err(GameError::NotYourTurn));
        assert_eq!(game.play(Player::X, 0), Ok(()));
        assert_eq!(game.play(Player::X, 4), Err(GameError::NotYourTurn));
        assert_eq!(game.play(Player::O, 4), Ok(()));
    }

    #[test]
    fn test_game_over_with_draw() {
        let mut game = Game::new();

        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 0), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 4), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 8), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 3), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 5), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 2), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 6), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 7), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 1), Ok(()));

        assert!(game.is_over());
    }

    #[test]
    fn test_game_over_with_winner_x() {
        let mut game = Game::new();

        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 0), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 4), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 1), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 5), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 2), Ok(()));

        assert!(game.is_over());
        assert_eq!(game.winner().unwrap(), Player::X);
    }

    #[test]
    fn test_game_over_with_winner_o() {
        let mut game = Game::new();

        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 0), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 4), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 1), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 5), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::X, 6), Ok(()));
        assert!(!game.is_over());
        assert_eq!(game.play(Player::O, 3), Ok(()));

        assert!(game.is_over());
        assert_eq!(game.winner().unwrap(), Player::O);
    }
}
