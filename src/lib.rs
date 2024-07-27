use std::{
    error::Error,
    fmt::{self, Display},
    ops::Not,
};

pub const VERSION_AND_GIT_HASH: &str = env!("VERSION_AND_GIT_HASH");

pub const LICENSE: &str = include_str!("../LICENSE");

#[derive(Debug, Clone)]
pub enum OthebotError {
    IllegalMove,
    LegalMovesNotComputed,
}

impl Error for OthebotError {}

impl Display for OthebotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OthebotError::IllegalMove => write!(f, "illegal move, you can't put your disc here"),
            OthebotError::LegalMovesNotComputed => write!(f, "INTERNAL ERROR: legal moves were not computed before calling a function that depends on legal moves.")
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Disc {
    White,
    Black,
    Empty,
}

impl Not for Disc {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Disc::White => Disc::Black,
            Disc::Black => Disc::White,
            // it shouldn't be called if `Disc` is `Empty` but if it did, don't
            // change because there is no opposite of `Empty`
            Disc::Empty => Disc::Empty,
        }
    }
}

impl Display for Disc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Disc::White => write!(f, "White"),
            Disc::Black => write!(f, "Black"),
            Disc::Empty => write!(f, "Empty"),
        }
    }
}

pub struct Board {
    discs: [Disc; 64],
}

impl Board {
    /// Create a new board with the starting layout
    pub const fn new() -> Board {
        use Disc::Black as B;
        use Disc::Empty as E;
        use Disc::White as W;
        Board {
            discs: [
                E, E, E, E, E, E, E, E, // This
                E, E, E, E, E, E, E, E, // is
                E, E, E, E, E, E, E, E, // to
                E, E, E, W, B, E, E, E, // trick
                E, E, E, B, W, E, E, E, // the
                E, E, E, E, E, E, E, E, // rust
                E, E, E, E, E, E, E, E, // formater
                E, E, E, E, E, E, E, E, // ;)
            ],
        }
    }

    /// Get the disc located at those X and Y coordinates, check if coordinates
    /// are in bounds
    #[inline]
    #[must_use]
    pub fn get_disc(&self, (col, row): (u8, u8)) -> Disc {
        assert!(col < 8);
        assert!(row < 8);
        // UNSAFE: we checked that they are in bounds
        unsafe { self.get_disc_unchecked(col, row) }
    }

    /// Get the disc at those X and Y coordiantes, don't check if they are in
    /// bounds are not.
    ///
    /// # Safety
    ///
    /// If either `x` or `y` are greater that 8, it will get the wrong disc, or
    /// panic. It is the responsability of the caller to check the coordinates
    /// are valid.
    #[inline]
    #[must_use]
    pub unsafe fn get_disc_unchecked(&self, col: u8, row: u8) -> Disc {
        self.discs[(row * 8 + col) as usize]
    }

    /// Change the disc at those coordinates, don't check if this move is legal.
    #[track_caller]
    fn change_disc(&mut self, (col, row): (u8, u8), disc: Disc) {
        assert!(col < 8);
        assert!(row < 8);
        // UNSAFE: we checked that they are in bounds
        let idx = (row * 8 + col) as usize;
        *self.discs.get_mut(idx).unwrap() = disc;
    }

    /// Returns the scores of the current board, in the tuple, white's score is
    /// first, and black's score is second
    pub fn scores(&self) -> (u8, u8) {
        let mut white = 0;
        let mut black = 0;
        for disc in self.discs {
            match disc {
                Disc::White => white += 1,
                Disc::Black => black += 1,
                Disc::Empty => {}
            }
        }
        (white, black)
    }

    /// Return the current legal moves for the `player` into a bitfield format.
    ///
    /// The first bit of the bitfield is the first disc at index 0 and the last
    /// bit is index 63.
    #[must_use]
    #[track_caller]
    pub fn legal_moves(&self, player: Disc) -> u64 {
        let mut bitfield = 0;

        if player == Disc::Empty {
            panic!("The player should not be an empty disc.")
        }

        let directions: [(i32, i32); 8] = [
            (-1, -1), // RIGHT UP
            (0, -1),  // UP
            (1, -1),  // LEFT-UP
            (-1, 0),  // RIGHT
            (1, 0),   // LEFT
            (-1, 1),  // LEFT-DOWN
            (0, -1),  // DOWN
            (1, 1),   // RIGHT-DOWN
        ];

        for y in 0..8 {
            for x in 0..8 {
                let idx = y * 8 + x;

                // The disc is already filed
                if self.discs[idx] != Disc::Empty {
                    continue;
                }

                for (dx, dy) in directions {
                    // coordinates of next disc in direction
                    let mut nx = x as i32 + dx;
                    let mut ny = y as i32 + dy;

                    // whetever a disc of the other color was present in the
                    // line of the direction
                    let mut captured = false;

                    while nx >= 0 && nx < 8 && ny >= 0 && ny < 8 {
                        let n_idx = (ny * 8 + nx) as usize;

                        if self.discs[n_idx] == Disc::Empty {
                            break;
                        }

                        if self.discs[n_idx] == player {
                            if captured {
                                // we already encountered an opposite disc, we
                                // know it is a good move
                                bitfield |= 1 << idx;
                            }
                            break;
                        }
                        // we encountered an opposite disc, so if later we
                        // encounter in the same direction a disc of player's
                        // color, it's a valid move
                        captured = true;
                        // update the coordinates to continue in this direction
                        nx += dx;
                        ny += dy;
                    }
                }
            }
        }

        bitfield
    }
}

impl Default for Board {
    #[inline]
    fn default() -> Self {
        Board::new()
    }
}

/// Converts an algebric notation like `a1`, `g8`, `b7` etc to `(0, 0)`,
/// `(6, 7)`, `(1, 6)`.
pub fn algebric2xy(pos: &str) -> Result<(u8, u8), OthebotError> {
    if pos.len() != 2 {
        return Err(OthebotError::IllegalMove);
    }

    let mut it = pos.chars();
    let col = it.next().unwrap() as u8;
    let row = it.next().unwrap() as u8;

    if !(b'a'..=b'h').contains(&col) {
        return Err(OthebotError::IllegalMove);
    }
    if !(b'1'..=b'8').contains(&row) {
        return Err(OthebotError::IllegalMove);
    }

    Ok((col - b'a', row - b'1'))
}

pub struct Game {
    board: Board,

    // TODO: if the given usernames are empty, don't use them, use instead their color.
    /// White player name
    white_player: String,

    /// Black player name
    black_player: String,

    /// Who's next turn?
    ///
    /// Note:
    ///
    /// `turn` cannot be `Disc::Empty`.
    turn: Disc,
    /// The legal moves of the current player (`turn` field).
    current_legal_moves: Option<u64>,
}

impl Game {
    pub fn new(white_player: impl Into<String>, black_player: impl Into<String>) -> Game {
        Game {
            board: Board::new(),
            white_player: white_player.into(),
            black_player: black_player.into(),
            turn: Disc::Black,
            current_legal_moves: None,
        }
    }

    pub fn turn(&self) -> Disc {
        self.turn
    }

    pub fn make_turn(&mut self, mov @ (row, col): (u8, u8)) -> Result<(), OthebotError> {
        // ensure the move is inside the legal moves.
        let idx = (row * 8 + col) as u64;
        let Some(legal_moves) = self.current_legal_moves else {
            return Err(OthebotError::LegalMovesNotComputed);
        };
        let mov_bitfield = 1 << idx;
        if legal_moves & mov_bitfield == 0 {
            return Err(OthebotError::IllegalMove);
        }
        self.board.change_disc(mov, self.turn);
        self.turn = !self.turn;

        self.current_legal_moves = None;
        Ok(())
    }

    #[inline]
    #[must_use]
    pub fn white_name(&self) -> &str {
        &self.white_player
    }

    #[inline]
    #[must_use]
    pub fn black_name(&self) -> &str {
        &self.black_player
    }

    #[inline]
    #[must_use]
    pub fn player_name(&self) -> &str {
        match self.turn {
            Disc::White => self.white_name(),
            Disc::Black => self.black_name(),
            Disc::Empty => unreachable!(),
        }
    }

    /// Renders the board game to stdout
    pub fn render(&self) -> Result<(), OthebotError> {
        // TODO: Add colors.
        let Some(legal_moves) = self.current_legal_moves else {
            return Err(OthebotError::LegalMovesNotComputed);
        };

        for row in 0..8 {
            print!("+---+---+---+---+---+---+---+---+");

            // print the scores
            if row == 7 {
                let (white_score, black_score) = self.board.scores();
                print!(
                    "    {}: {}  {}: {}",
                    self.black_name(),
                    black_score,
                    self.white_name(),
                    white_score,
                );
            }

            println!();

            for col in 0..8 {
                let idx = row * 8 + col;
                let is_legal_move = (1 << idx) & legal_moves != 0;
                let disc = self.board.discs[idx];
                print!("| ");
                match disc {
                    Disc::White => print!("W"),
                    Disc::Black => print!("B"),
                    Disc::Empty if is_legal_move => print!("•"),
                    Disc::Empty => print!(" "),
                }
                print!(" ");
            }

            print!("| {}", row + 1);

            // print the score
            if row == 6 {
                print!("  SCORES:");
            }

            println!();
        }
        println!("+---+---+---+---+---+---+---+---+");
        println!("  a   b   c   d   e   f   g   h");

        Ok(())
    }

    /// Compute and store the legal moves of the current player.
    pub fn legal_moves(&mut self) {
        self.current_legal_moves = Some(self.board.legal_moves(self.turn()));
    }
}
