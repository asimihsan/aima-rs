#[derive(Debug, thiserror::Error)]
pub enum ConnectFourError {
    #[error("invalid column: {0}")]
    InvalidColumn(usize),

    #[error("invalid row: {0}")]
    InvalidRow(usize),

    #[error("column is full: {0}")]
    ColumnFull(usize),

    #[error("column is empty: {0}")]
    ColumnEmpty(usize),

    #[error("column is not yours: {0}")]
    ColumnNotYours(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cell {
    Empty,
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Board<const WIDTH: usize, const HEIGHT: usize> {
    pub cells: [[Cell; WIDTH]; HEIGHT],
}

impl<const WIDTH: usize, const HEIGHT: usize> Default for Board<WIDTH, HEIGHT> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Board<WIDTH, HEIGHT> {
    pub fn new() -> Self {
        Self {
            cells: [[Cell::Empty; WIDTH]; HEIGHT],
        }
    }

    pub fn get(&self, col: usize, row: usize) -> Result<Cell, ConnectFourError> {
        if col >= WIDTH {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        if row >= HEIGHT {
            return Err(ConnectFourError::InvalidRow(row));
        }
        Ok(self.cells[row][col])
    }

    pub fn can_insert(&self, col: usize) -> Result<(), ConnectFourError> {
        if col >= WIDTH {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        for row in (0..HEIGHT).rev() {
            if self.cells[row][col] == Cell::Empty {
                return Ok(());
            }
        }
        Err(ConnectFourError::ColumnFull(col))
    }

    // insert will insert a piece into the board. It will return None if the column is full.
    // This inserts into the first empty cell in the column, going from the bottom up.
    pub fn insert(&mut self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        match self.can_insert(col) {
            Ok(()) => {
                for row in (0..HEIGHT).rev() {
                    if self.cells[row][col] == Cell::Empty {
                        self.cells[row][col] = match player {
                            Player::Player1 => Cell::Player1,
                            Player::Player2 => Cell::Player2,
                        };
                        return Ok(());
                    }
                }
                unreachable!()
            }
            Err(e) => Err(e),
        }
    }

    pub fn can_pop(&self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        if col >= WIDTH {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        for row in (0..HEIGHT).rev() {
            if self.cells[row][col] == Cell::Empty {
                continue;
            }
            if self.cells[row][col]
                == match player {
                    Player::Player1 => Cell::Player1,
                    Player::Player2 => Cell::Player2,
                }
            {
                return Ok(());
            }
        }
        Err(ConnectFourError::ColumnEmpty(col))
    }

    // pop will remove a piece from the board. It will return None if the column is empty.
    // This removes the first non-empty cell in the column, going from the bottom up.
    // This will shift down all the pieces above it. This is used for the popout variant.
    //
    // You can only pop from a column if the bottom piece is yours.
    pub fn pop(&mut self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        match self.can_pop(col, player) {
            Ok(()) => {
                for row in (0..HEIGHT).rev() {
                    if self.cells[row][col] != Cell::Empty {
                        self.cells[row][col] = Cell::Empty;
                        return Ok(());
                    }
                }
                unreachable!()
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    Insert(usize),
    Pop(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerMove {
    Player1(Move),
    Player2(Move),
}

pub fn get_legal_moves<const WIDTH: usize, const HEIGHT: usize>(
    board: &Board<WIDTH, HEIGHT>,
    player: Player,
) -> Vec<PlayerMove> {
    let mut moves = Vec::new();
    for col in 0..WIDTH {
        if board.can_insert(col).is_ok() {
            match player {
                Player::Player1 => moves.push(PlayerMove::Player1(Move::Insert(col))),
                Player::Player2 => moves.push(PlayerMove::Player2(Move::Insert(col))),
            }
        }
        if board.can_pop(col, player).is_ok() {
            match player {
                Player::Player1 => moves.push(PlayerMove::Player1(Move::Pop(col))),
                Player::Player2 => moves.push(PlayerMove::Player2(Move::Pop(col))),
            }
        }
    }
    moves
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_starts_empty() {
        let mut board = Board::<7, 6>::new();
        for row in 0..6 {
            for col in 0..7 {
                let get = board
                    .get(col, row)
                    .expect(format!("col: {}, row: {}", col, row).as_str());
                assert_eq!(get, Cell::Empty, "col: {}, row: {}", col, row);
            }
        }
    }

    #[test]
    fn test_at_start_all_inserts_no_pops_legal() {
        let board = Board::<7, 6>::new();
        for col in 0..7 {
            assert!(board.can_insert(col).is_ok());
            assert!(board.can_pop(col, Player::Player1).is_err());
            assert!(board.can_pop(col, Player::Player2).is_err());
        }

        for player in &[Player::Player1, Player::Player2] {
            let legal_moves = get_legal_moves(&board, *player);
            assert_eq!(legal_moves.len(), 7);
            for col in 0..7 {
                match player {
                    Player::Player1 => {
                        assert!(legal_moves.contains(&PlayerMove::Player1(Move::Insert(col))));
                        assert!(!legal_moves.contains(&PlayerMove::Player1(Move::Pop(col))));
                    }
                    Player::Player2 => {
                        assert!(legal_moves.contains(&PlayerMove::Player2(Move::Insert(col))));
                        assert!(!legal_moves.contains(&PlayerMove::Player2(Move::Pop(col))));
                    }
                }
            }
        }
    }

    #[test]
    fn test_board_one_insert() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1).expect("insert failed");

        for row in 0..6 {
            for col in 0..7 {
                let get = board
                    .get(col, row)
                    .unwrap_or_else(|_| panic!("col: {}, row: {}", col, row));
                if col == 0 && row == 5 {
                    assert_eq!(get, Cell::Player1, "col: {}, row: {}", col, row);
                } else {
                    assert_eq!(get, Cell::Empty, "col: {}, row: {}", col, row);
                }
            }
        }
    }

    #[test]
    fn test_board_one_insert_then_pop() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1).expect("insert failed");
        board.pop(0, Player::Player1).expect("pop failed");

        for row in 0..6 {
            for col in 0..7 {
                let get = board
                    .get(col, row)
                    .expect(format!("col: {}, row: {}", col, row).as_str());
                assert_eq!(get, Cell::Empty, "col: {}, row: {}", col, row);
            }
        }
    }
}
