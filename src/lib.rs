//! Rules engine for Master Ludo.
//!
//! The game model is independent from rendering so it can be tested
//! without opening a graphics window.

pub const TRACK_LEN: usize = 52;
pub const TOKEN_COUNT: usize = 4;
pub const YARD: i8 = -1;
pub const FINISHED: i8 = 58;
pub const LAST_HOME_STEP: i8 = 57;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos {
    pub x: i8,
    pub y: i8,
}

impl Pos {
    pub const fn new(x: i8, y: i8) -> Self {
        Self { x, y }
    }
}

/// The shared 52-square outer route, clockwise around the board.
pub const TRACK: [Pos; TRACK_LEN] = [
    Pos::new(1, 6), Pos::new(2, 6), Pos::new(3, 6), Pos::new(4, 6), Pos::new(5, 6),
    Pos::new(6, 5), Pos::new(6, 4), Pos::new(6, 3), Pos::new(6, 2), Pos::new(6, 1),
    Pos::new(6, 0), Pos::new(7, 0), Pos::new(8, 0),
    Pos::new(8, 1), Pos::new(8, 2), Pos::new(8, 3), Pos::new(8, 4), Pos::new(8, 5),
    Pos::new(9, 6), Pos::new(10, 6), Pos::new(11, 6), Pos::new(12, 6), Pos::new(13, 6),
    Pos::new(14, 6), Pos::new(14, 7), Pos::new(14, 8),
    Pos::new(13, 8), Pos::new(12, 8), Pos::new(11, 8), Pos::new(10, 8), Pos::new(9, 8),
    Pos::new(8, 9), Pos::new(8, 10), Pos::new(8, 11), Pos::new(8, 12), Pos::new(8, 13),
    Pos::new(8, 14), Pos::new(7, 14), Pos::new(6, 14),
    Pos::new(6, 13), Pos::new(6, 12), Pos::new(6, 11), Pos::new(6, 10), Pos::new(6, 9),
    Pos::new(5, 8), Pos::new(4, 8), Pos::new(3, 8), Pos::new(2, 8), Pos::new(1, 8),
    Pos::new(0, 8), Pos::new(0, 7), Pos::new(0, 6),
];

/// Start squares and star squares are protected from captures.
pub const SAFE_SQUARES: [usize; 8] = [0, 8, 13, 21, 26, 34, 39, 47];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerColor {
    Red,
    Green,
    Yellow,
    Blue,
}

impl PlayerColor {
    pub const ALL: [Self; 4] = [Self::Red, Self::Green, Self::Yellow, Self::Blue];

    pub const fn index(self) -> usize {
        match self {
            Self::Red => 0,
            Self::Green => 1,
            Self::Yellow => 2,
            Self::Blue => 3,
        }
    }

    pub const fn start_index(self) -> usize {
        self.index() * 13
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Red => "RED",
            Self::Green => "GREEN",
            Self::Yellow => "YELLOW",
            Self::Blue => "BLUE",
        }
    }

    pub const fn home_lane(self) -> [Pos; 6] {
        match self {
            Self::Red => [
                Pos::new(1, 7), Pos::new(2, 7), Pos::new(3, 7),
                Pos::new(4, 7), Pos::new(5, 7), Pos::new(6, 7),
            ],
            Self::Green => [
                Pos::new(7, 1), Pos::new(7, 2), Pos::new(7, 3),
                Pos::new(7, 4), Pos::new(7, 5), Pos::new(7, 6),
            ],
            Self::Yellow => [
                Pos::new(13, 7), Pos::new(12, 7), Pos::new(11, 7),
                Pos::new(10, 7), Pos::new(9, 7), Pos::new(8, 7),
            ],
            Self::Blue => [
                Pos::new(7, 13), Pos::new(7, 12), Pos::new(7, 11),
                Pos::new(7, 10), Pos::new(7, 9), Pos::new(7, 8),
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Player {
    pub color: PlayerColor,
    /// -1 means in the yard, 0..=51 means on the shared route,
    /// 52..=57 means in the home lane, and 58 means finished.
    pub tokens: [i8; TOKEN_COUNT],
}

impl Player {
    fn new(color: PlayerColor) -> Self {
        Self {
            color,
            tokens: [YARD; TOKEN_COUNT],
        }
    }

    pub fn finished_count(&self) -> usize {
        self.tokens.iter().filter(|&&p| p == FINISHED).count()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    AwaitRoll,
    AwaitToken,
    GameOver,
}

#[derive(Clone, Debug)]
pub struct MoveReport {
    pub token_index: usize,
    pub captured: usize,
    pub reached_home: bool,
    pub extra_turn: bool,
}

#[derive(Clone, Debug)]
pub struct Game {
    pub players: Vec<Player>,
    pub current_player: usize,
    pub dice: Option<u8>,
    pub phase: Phase,
    pub consecutive_sixes: u8,
    pub winner: Option<PlayerColor>,
    pub message: String,
}

impl Game {
    /// Creates an offline game with 2, 3, or 4 players.
    pub fn new(player_count: usize) -> Self {
        let count = player_count.clamp(2, 4);
        let players = PlayerColor::ALL
            .into_iter()
            .take(count)
            .map(Player::new)
            .collect();

        Self {
            players,
            current_player: 0,
            dice: None,
            phase: Phase::AwaitRoll,
            consecutive_sixes: 0,
            winner: None,
            message: "Roll a 6 to bring a token out of the yard.".to_owned(),
        }
    }

    pub fn current_color(&self) -> PlayerColor {
        self.players[self.current_player].color
    }

    /// Returns every token that can legally move for the current dice value.
    pub fn legal_moves(&self) -> Vec<usize> {
        let Some(dice) = self.dice else {
            return Vec::new();
        };
        if self.phase != Phase::AwaitToken {
            return Vec::new();
        }

        self.players[self.current_player]
            .tokens
            .iter()
            .enumerate()
            .filter_map(|(index, &progress)| {
                let can_move = if progress == YARD {
                    dice == 6
                } else if (0..=LAST_HOME_STEP).contains(&progress) {
                    progress + dice as i8 <= LAST_HOME_STEP
                } else {
                    false
                };
                can_move.then_some(index)
            })
            .collect()
    }

    /// Applies a dice roll. The argument is supplied by the UI, which makes
    /// the rules engine deterministic and straightforward to test.
    pub fn roll_dice(&mut self, roll: u8) -> Result<(), &'static str> {
        if self.phase != Phase::AwaitRoll {
            return Err("It is not time to roll the dice.");
        }
        if !(1..=6).contains(&roll) {
            return Err("Dice value must be between 1 and 6.");
        }

        if roll == 6 {
            self.consecutive_sixes += 1;
            if self.consecutive_sixes == 3 {
                self.dice = None;
                self.consecutive_sixes = 0;
                self.advance_turn();
                self.message = "Three sixes. Turn passed to the next player.".to_owned();
                return Ok(());
            }
        } else {
            self.consecutive_sixes = 0;
        }

        self.dice = Some(roll);
        self.phase = Phase::AwaitToken;
        let legal = self.legal_moves();
        if legal.is_empty() {
            self.dice = None;
            self.phase = Phase::AwaitRoll;
            if roll == 6 {
                self.message = format!(
                    "{} rolled a 6 but has no legal move. Roll again.",
                    self.current_color().label()
                );
            } else {
                let name = self.current_color().label();
                self.advance_turn();
                self.message = format!("{name} has no legal move. Next player's turn.");
            }
        } else {
            self.message = format!(
                "{} rolled {roll}. Tap a highlighted token.",
                self.current_color().label()
            );
        }
        Ok(())
    }

    /// Moves one of the current player's tokens and resolves captures/wins.
    pub fn move_token(&mut self, token_index: usize) -> Result<MoveReport, &'static str> {
        if self.phase != Phase::AwaitToken {
            return Err("Roll the dice before moving a token.");
        }
        if token_index >= TOKEN_COUNT {
            return Err("Invalid token index.");
        }
        if !self.legal_moves().contains(&token_index) {
            return Err("That token cannot move with this dice value.");
        }

        let roll = self.dice.ok_or("Dice value is missing.")?;
        let current = self.current_player;
        let color = self.players[current].color;
        let old_progress = self.players[current].tokens[token_index];
        let new_progress = if old_progress == YARD {
            0
        } else {
            old_progress + roll as i8
        };
        self.players[current].tokens[token_index] = new_progress;

        let mut captured = 0;
        if (0..=51).contains(&new_progress) {
            let target_index = (color.start_index() + new_progress as usize) % TRACK_LEN;
            if !SAFE_SQUARES.contains(&target_index) {
                for player_index in 0..self.players.len() {
                    if player_index == current {
                        continue;
                    }
                    let opponent_start = self.players[player_index].color.start_index();
                    for opponent_token in &mut self.players[player_index].tokens {
                        if (0..=51).contains(opponent_token) {
                            let opponent_index =
                                (opponent_start + *opponent_token as usize) % TRACK_LEN;
                            if opponent_index == target_index {
                                *opponent_token = YARD;
                                captured += 1;
                            }
                        }
                    }
                }
            }
        }

        let reached_home = new_progress == LAST_HOME_STEP;
        let all_tokens_at_finish = self.players[current]
            .tokens
            .iter()
            .all(|&progress| progress == FINISHED || progress == LAST_HOME_STEP);
        if reached_home {
            self.players[current].tokens[token_index] = FINISHED;
        }
        let actually_won = all_tokens_at_finish
            && self.players[current]
                .tokens
                .iter()
                .all(|&progress| progress == FINISHED);
        let extra_turn = roll == 6 || captured > 0 || reached_home;

        self.dice = None;
        if actually_won {
            self.winner = Some(color);
            self.phase = Phase::GameOver;
            self.message = format!("{} wins the game!", color.label());
        } else if extra_turn {
            self.phase = Phase::AwaitRoll;
            if roll != 6 {
                self.consecutive_sixes = 0;
            }
            self.message = if captured > 0 {
                format!("{} captured {captured} token(s). Roll again!", color.label())
            } else if reached_home {
                format!("{} brought a token home. Roll again!", color.label())
            } else {
                format!("{} rolled a 6. Roll again!", color.label())
            };
        } else {
            self.advance_turn();
            self.message = format!("{} moved. Next player's turn.", color.label());
        }

        Ok(MoveReport {
            token_index,
            captured,
            reached_home,
            extra_turn,
        })
    }

    fn advance_turn(&mut self) {
        self.current_player = (self.current_player + 1) % self.players.len();
        self.dice = None;
        self.phase = Phase::AwaitRoll;
        self.consecutive_sixes = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_has_52_unique_squares() {
        for i in 0..TRACK_LEN {
            for j in (i + 1)..TRACK_LEN {
                assert_ne!(TRACK[i], TRACK[j], "duplicate track squares at {i} and {j}");
            }
        }
    }

    #[test]
    fn token_needs_six_to_leave_yard() {
        let mut game = Game::new(2);
        game.roll_dice(5).unwrap();
        assert_eq!(game.phase, Phase::AwaitRoll);
        assert_eq!(game.current_player, 1);
        assert!(game.players[0].tokens.iter().all(|&p| p == YARD));
    }

    #[test]
    fn six_brings_a_token_out_and_grants_another_turn() {
        let mut game = Game::new(2);
        game.roll_dice(6).unwrap();
        let report = game.move_token(0).unwrap();
        assert_eq!(game.players[0].tokens[0], 0);
        assert!(report.extra_turn);
        assert_eq!(game.current_player, 0);
        assert_eq!(game.phase, Phase::AwaitRoll);
    }

    #[test]
    fn exact_roll_is_required_to_finish() {
        let mut game = Game::new(2);
        game.players[0].tokens[0] = 56;
        game.roll_dice(2).unwrap();
        assert!(game.legal_moves().is_empty());
        assert_eq!(game.current_player, 1);
        assert_eq!(game.players[0].tokens[0], 56);
    }

    #[test]
    fn capture_sends_opponent_token_back_to_yard() {
        let mut game = Game::new(2);
        game.players[0].tokens[0] = 4;
        // Green starts at route index 13; progress 44 puts it on route index 5.
        game.players[1].tokens[0] = 44;
        game.roll_dice(1).unwrap();
        let report = game.move_token(0).unwrap();
        assert_eq!(report.captured, 1);
        assert_eq!(game.players[1].tokens[0], YARD);
        assert_eq!(game.current_player, 0);
    }

    #[test]
    fn safe_squares_prevent_capture() {
        let mut game = Game::new(2);
        game.players[0].tokens[0] = 7;
        // Green progress 47 is on shared route index 8, a safe square.
        game.players[1].tokens[0] = 47;
        game.roll_dice(1).unwrap();
        let report = game.move_token(0).unwrap();
        assert_eq!(report.captured, 0);
        assert_eq!(game.players[1].tokens[0], 47);
    }

    #[test]
    fn all_four_tokens_must_reach_home_to_win() {
        let mut game = Game::new(2);
        game.players[0].tokens = [FINISHED, FINISHED, FINISHED, 56];
        game.roll_dice(1).unwrap();
        game.move_token(3).unwrap();
        assert_eq!(game.phase, Phase::GameOver);
        assert_eq!(game.winner, Some(PlayerColor::Red));
    }

    #[test]
    fn third_consecutive_six_passes_the_turn_without_moving() {
        let mut game = Game::new(2);
        game.roll_dice(6).unwrap();
        game.move_token(0).unwrap();
        game.roll_dice(6).unwrap();
        game.move_token(1).unwrap();
        game.roll_dice(6).unwrap();
        assert_eq!(game.current_player, 1);
        assert_eq!(game.phase, Phase::AwaitRoll);
        assert_eq!(game.players[0].tokens[2], YARD);
    }
}
