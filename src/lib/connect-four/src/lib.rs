use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
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
    Player(Player),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Player {
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

    pub fn get_col(&self, col: usize) -> Result<[Cell; HEIGHT], ConnectFourError> {
        if col >= WIDTH {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        let mut result = [Cell::Empty; HEIGHT];
        for row in 0..HEIGHT {
            result[row] = self.cells[row][col];
        }
        Ok(result)
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
                        self.cells[row][col] = Cell::Player(player);
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
        let col_cells = self.get_col(col)?;
        if col_cells[HEIGHT - 1] == Cell::Empty {
            return Err(ConnectFourError::ColumnEmpty(col));
        }
        if col_cells[HEIGHT - 1] != Cell::Player(player) {
            return Err(ConnectFourError::ColumnNotYours(col));
        }
        Ok(())
    }

    // pop will remove a piece from the board. It will return None if the column is empty.
    // This removes the first non-empty cell in the column, going from the bottom up.
    // This will shift down all the pieces above it. This is used for the popout variant.
    //
    // You can only pop from a column if the bottom piece is yours.
    pub fn pop(&mut self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        match self.can_pop(col, player) {
            Ok(()) => {
                // for this column, copy the i+1 higher element down to i, in reverse order
                for row in (0..HEIGHT - 1).rev() {
                    self.cells[row + 1][col] = self.cells[row][col];
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Move {
    Insert(usize),
    Pop(usize),
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Insert(col) => write!(f, "Insert({})", col),
            Move::Pop(col) => write!(f, "Pop({})", col),
        }
    }
}

pub fn get_legal_moves<const WIDTH: usize, const HEIGHT: usize>(
    board: &Board<WIDTH, HEIGHT>,
    player: Player,
) -> Vec<Move> {
    let mut moves = Vec::new();
    for col in 0..WIDTH {
        if board.can_insert(col).is_ok() {
            moves.push(Move::Insert(col));
        }
        if board.can_pop(col, player).is_ok() {
            moves.push(Move::Pop(col));
        }
    }
    moves
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalPosition {
    IsTerminalWin(Player),
    IsTerminalDraw,
    IsNotTerminal,
}

pub fn is_terminal_position<const WIDTH: usize, const HEIGHT: usize>(
    board: &Board<WIDTH, HEIGHT>,
) -> TerminalPosition {
    // check for a win
    for row in 0..HEIGHT {
        for col in 0..WIDTH {
            let cell1 = board.get(col, row).unwrap();
            if cell1 == Cell::Empty {
                continue;
            }
            let player = match cell1 {
                Cell::Player(p) => p,
                Cell::Empty => unreachable!(),
            };

            // check horizontal
            if col + 3 < WIDTH {
                let cell2 = board.get(col + 1, row).unwrap();
                let cell3 = board.get(col + 2, row).unwrap();
                let cell4 = board.get(col + 3, row).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check vertical
            if row + 3 < HEIGHT {
                let cell2 = board.get(col, row + 1).unwrap();
                let cell3 = board.get(col, row + 2).unwrap();
                let cell4 = board.get(col, row + 3).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check diagonal down
            if col + 3 < WIDTH && row + 3 < HEIGHT {
                let cell2 = board.get(col + 1, row + 1).unwrap();
                let cell3 = board.get(col + 2, row + 2).unwrap();
                let cell4 = board.get(col + 3, row + 3).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check diagonal up
            if col + 3 < WIDTH && row >= 3 {
                let cell2 = board.get(col + 1, row - 1).unwrap();
                let cell3 = board.get(col + 2, row - 2).unwrap();
                let cell4 = board.get(col + 3, row - 3).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }
        }
    }

    // check for a draw
    if get_legal_moves(board, Player::Player1).is_empty() {
        TerminalPosition::IsTerminalDraw
    } else {
        TerminalPosition::IsNotTerminal
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn all_cells_empty<const WIDTH: usize, const HEIGHT: usize>(
        board: &Board<WIDTH, HEIGHT>,
    ) -> bool {
        for row in 0..HEIGHT {
            for col in 0..WIDTH {
                if board.get(col, row).unwrap() != Cell::Empty {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn test_board_starts_empty() {
        let board = Board::<7, 6>::new();
        for row in 0..6 {
            for col in 0..7 {
                let get = board
                    .get(col, row)
                    .unwrap_or_else(|_| panic!("col: {}, row: {}", col, row));
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
                assert!(legal_moves.contains(&Move::Insert(col)));
                assert!(!legal_moves.contains(&Move::Pop(col)));
            }
        }
    }

    #[test]
    fn test_insert_then_invalid_pop_returns_error() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1).expect("insert failed");
        assert_eq!(
            board.can_pop(0, Player::Player2),
            Err(ConnectFourError::ColumnNotYours(0)),
            "should not be able to pop"
        );
    }

    #[test]
    fn test_empty_board_all_pops_illegal_due_to_column_empty() {
        let board = Board::<7, 6>::new();
        for col in 0..7 {
            assert!(board.can_pop(col, Player::Player1) == Err(ConnectFourError::ColumnEmpty(col)));
            assert!(board.can_pop(col, Player::Player2) == Err(ConnectFourError::ColumnEmpty(col)));
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
                    assert_eq!(
                        get,
                        Cell::Player(Player::Player1),
                        "col: {}, row: {}",
                        col,
                        row
                    );
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

        assert!(all_cells_empty(&board));
    }

    #[test]
    fn test_board_two_inserts_then_pop() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player2).expect("insert failed");
        let col0 = board.get_col(0).unwrap();
        assert_eq!(
            col0,
            [
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Player(Player::Player2)
            ]
        );

        board.insert(0, Player::Player1).expect("insert failed");
        let col0 = board.get_col(0).unwrap();
        assert_eq!(
            col0,
            [
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Player(Player::Player1),
                Cell::Player(Player::Player2),
            ]
        );

        board.pop(0, Player::Player2).expect("pop failed");
        let col0 = board.get_col(0).unwrap();
        assert_eq!(
            col0,
            [
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Empty,
                Cell::Player(Player::Player1),
            ]
        );
    }

    #[test]
    fn test_empty_board_is_not_terminal() {
        let board = Board::<7, 6>::new();
        assert_eq!(
            is_terminal_position(&board),
            TerminalPosition::IsNotTerminal
        );
    }

    #[test]
    fn test_is_terminal_horizontal_win() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1).expect("insert failed");
        board.insert(1, Player::Player1).expect("insert failed");
        board.insert(2, Player::Player1).expect("insert failed");
        board.insert(3, Player::Player1).expect("insert failed");
        assert_eq!(
            is_terminal_position(&board),
            TerminalPosition::IsTerminalWin(Player::Player1)
        );
    }

    #[test]
    fn test_is_terminal_vertical_win() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1).expect("insert failed");
        board.insert(0, Player::Player1).expect("insert failed");
        board.insert(0, Player::Player1).expect("insert failed");
        board.insert(0, Player::Player1).expect("insert failed");
        assert_eq!(
            is_terminal_position(&board),
            TerminalPosition::IsTerminalWin(Player::Player1)
        );
    }

    #[test]
    fn test_is_terminal_diag_win() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1).expect("insert failed");
        board.insert(1, Player::Player2).expect("insert failed");
        board.insert(1, Player::Player1).expect("insert failed");
        board.insert(2, Player::Player2).expect("insert failed");
        board.insert(2, Player::Player2).expect("insert failed");
        board.insert(2, Player::Player1).expect("insert failed");
        board.insert(3, Player::Player2).expect("insert failed");
        board.insert(3, Player::Player2).expect("insert failed");
        board.insert(3, Player::Player2).expect("insert failed");
        board.insert(3, Player::Player1).expect("insert failed");
        assert_eq!(
            is_terminal_position(&board),
            TerminalPosition::IsTerminalWin(Player::Player1)
        );
    }

    fn vec_of_player() -> impl Strategy<Value = Vec<Player>> {
        prop::collection::vec(
            prop_oneof![Just(Player::Player1), Just(Player::Player2)],
            1..6,
        )
    }

    proptest! {
        #[test]
        fn test_invalid_row_returns_error(
            col in 0..7usize,
            row in 6..usize::MAX,
        ) {
            let board = Board::<7, 6>::new();
            assert_eq!(
                board.get(col, row),
                Err(ConnectFourError::InvalidRow(row)),
                "should not be able to get row"
            );
        }

        #[test]
        fn test_invalid_col_returns_error(
            col in 7..usize::MAX,
            row in 0..6usize,
        ) {
            let board = Board::<7, 6>::new();
            assert_eq!(
                board.get(col, row),
                Err(ConnectFourError::InvalidColumn(col)),
                "should not be able to get col"
            );
        }

        #[test]
        fn test_board_insert_then_pop_means_empty(
            col in 0..7usize,
            player in prop_oneof![Just(Player::Player1), Just(Player::Player2)],
        ) {
            let mut board = Board::<7, 6>::new();
            board.insert(col, player).expect("insert failed");
            board.pop(col, player).expect("pop failed");
            assert!(all_cells_empty(&board));
        }

        #[test]
        fn test_column_full_then_insert_means_error(
            col in 0..7usize,
            player in prop_oneof![Just(Player::Player1), Just(Player::Player2)],
        ) {
            let mut board = Board::<7, 6>::new();
            for _ in 0..6 {
                board.insert(col, player).expect("insert failed");
            }
            assert_eq!(
                board.can_insert(col),
                Err(ConnectFourError::ColumnFull(col)),
                "should not be able to insert"
            );
        }

        #[test]
        fn test_board_many_inserts_then_one_pop(
            col in 0..7usize,
            player in vec_of_player(),
        ) {
            let mut board = Board::<7, 6>::new();
            for p in &player {
                board.insert(col, *p).expect("insert failed");
            }

            let initial_col = board.get_col(col).expect("get_col failed");

            let first_player = &player.first().unwrap();
            board.pop(col, **first_player).expect("pop failed");
            let final_col = board.get_col(col).expect("get_col failed");

            let initial_col_non_empty = initial_col.iter().filter(|c| **c != Cell::Empty).collect::<Vec<_>>();
            let final_col_non_empty = final_col.iter().filter(|c| **c != Cell::Empty).collect::<Vec<_>>();

            // final_col should have one more empty cell that initial_col
            assert_eq!(initial_col_non_empty.len(), final_col_non_empty.len() + 1);

            // initial_col_non_empty except last element is same as final
            assert_eq!(initial_col_non_empty[..initial_col_non_empty.len() - 1], final_col_non_empty[..]);
        }
    }
}
