/*
 * Copyright 2023 Asim Ihsan
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cell {
    Empty,
    Player(Player),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    Player1,
    Player2,
}

impl Player {
    pub fn other(&mut self) {
        match self {
            Player::Player1 => *self = Player::Player2,
            Player::Player2 => *self = Player::Player1,
        }
    }
}

impl std::fmt::Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::Player1 => write!(f, "Player 1"),
            Player::Player2 => write!(f, "Player 2"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    pub cells: Vec<Vec<Cell>>,
    width: usize,
    height: usize,
}

// print out cells, and row and column numbers which start at 0.
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = self.cells[0].len();
        let height = self.cells.len();
        let mut s = String::with_capacity((width * 2 + 1) * (height + 1));

        // print column numbers. recall there will be row numbers on the left.
        for col in 0..width {
            if col == 0 {
                s.push_str("  ");
            }

            s.push_str(&format!("{}", col));
            if col == width - 1 {
                s.push('\n');
            } else {
                s.push(' ');
            }
        }

        for row in 0..height {
            // print row numbers
            s.push_str(&format!("{} ", row));

            for col in 0..width {
                let cell = self.cells[row][col];
                let c = match cell {
                    Cell::Empty => '.',
                    Cell::Player(Player::Player1) => '1',
                    Cell::Player(Player::Player2) => '2',
                };
                s.push(c);
                if col < width - 1 {
                    s.push(' ');
                }
            }
            if row < height - 1 {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let mut cells = Vec::with_capacity(height);
        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(Cell::Empty);
            }
            cells.push(row);
        }
        Self {
            cells,
            width,
            height,
        }
    }

    pub fn get(&self, col: usize, row: usize) -> Result<Cell, ConnectFourError> {
        if col >= self.width {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        if row >= self.height {
            return Err(ConnectFourError::InvalidRow(row));
        }
        Ok(self.cells[row][col])
    }

    pub fn get_col(&self, col: usize) -> Result<Vec<Cell>, ConnectFourError> {
        if col >= self.width {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        let result = (0..self.height).map(|row| self.cells[row][col]).collect();
        Ok(result)
    }

    pub fn can_insert(&self, col: usize) -> Result<(), ConnectFourError> {
        if col >= self.width {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        for row in (0..self.height).rev() {
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
                for row in (0..self.height).rev() {
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
        if col >= self.width {
            return Err(ConnectFourError::InvalidColumn(col));
        }
        let col_cells = self.get_col(col)?;
        if col_cells[self.height - 1] == Cell::Empty {
            return Err(ConnectFourError::ColumnEmpty(col));
        }
        if col_cells[self.height - 1] != Cell::Player(player) {
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
                for row in (0..self.height - 1).rev() {
                    self.cells[row + 1][col] = self.cells[row][col];
                }
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Move {
    Insert(usize),
    Pop(usize),
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move::Insert(col) => write!(f, "Insert({})", col),
            Move::Pop(col) => write!(f, "Pop({})", col),
        }
    }
}

pub fn get_legal_moves(board: &Board, player: Player) -> Vec<Move> {
    let mut moves = Vec::new();
    for col in 0..board.width {
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

pub fn is_terminal_position(board: &Board) -> TerminalPosition {
    // check for a win
    for row in 0..board.height {
        for col in 0..board.width {
            let cell1 = board.get(col, row).unwrap();
            if cell1 == Cell::Empty {
                continue;
            }
            let player = match cell1 {
                Cell::Player(p) => p,
                Cell::Empty => unreachable!(),
            };

            // check horizontal
            if col + 3 < board.width {
                let cell2 = board.get(col + 1, row).unwrap();
                let cell3 = board.get(col + 2, row).unwrap();
                let cell4 = board.get(col + 3, row).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check vertical
            if row + 3 < board.height {
                let cell2 = board.get(col, row + 1).unwrap();
                let cell3 = board.get(col, row + 2).unwrap();
                let cell4 = board.get(col, row + 3).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check diagonal down
            if col + 3 < board.width && row + 3 < board.height {
                let cell2 = board.get(col + 1, row + 1).unwrap();
                let cell3 = board.get(col + 2, row + 2).unwrap();
                let cell4 = board.get(col + 3, row + 3).unwrap();
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check diagonal up
            if col + 3 < board.width && row >= 3 {
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

    fn all_cells_empty(board: &Board) -> bool {
        for row in 0..board.height {
            for col in 0..board.width {
                if board.get(col, row).unwrap() != Cell::Empty {
                    return false;
                }
            }
        }
        true
    }

    #[test]
    fn test_board_starts_empty() {
        let board = Board::new(7, 6);
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
        let board = Board::new(7, 6);
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
        let mut board = Board::new(7, 6);
        board.insert(0, Player::Player1).expect("insert failed");
        assert_eq!(
            board.can_pop(0, Player::Player2),
            Err(ConnectFourError::ColumnNotYours(0)),
            "should not be able to pop"
        );
    }

    #[test]
    fn test_empty_board_all_pops_illegal_due_to_column_empty() {
        let board = Board::new(7, 6);
        for col in 0..7 {
            assert!(board.can_pop(col, Player::Player1) == Err(ConnectFourError::ColumnEmpty(col)));
            assert!(board.can_pop(col, Player::Player2) == Err(ConnectFourError::ColumnEmpty(col)));
        }
    }

    #[test]
    fn test_board_one_insert() {
        let mut board = Board::new(7, 6);
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
        let mut board = Board::new(7, 6);
        board.insert(0, Player::Player1).expect("insert failed");
        board.pop(0, Player::Player1).expect("pop failed");

        assert!(all_cells_empty(&board));
    }

    #[test]
    fn test_board_two_inserts_then_pop() {
        let mut board = Board::new(7, 6);
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
        let board = Board::new(7, 6);
        assert_eq!(
            is_terminal_position(&board),
            TerminalPosition::IsNotTerminal
        );
    }

    #[test]
    fn test_is_terminal_horizontal_win() {
        let mut board = Board::new(7, 6);
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
        let mut board = Board::new(7, 6);
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
        let mut board = Board::new(7, 6);
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
            let board = Board::new(7, 6);
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
            let board = Board::new(7, 6);
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
            let mut board = Board::new(7, 6);
            board.insert(col, player).expect("insert failed");
            board.pop(col, player).expect("pop failed");
            assert!(all_cells_empty(&board));
        }

        #[test]
        fn test_column_full_then_insert_means_error(
            col in 0..7usize,
            player in prop_oneof![Just(Player::Player1), Just(Player::Player2)],
        ) {
            let mut board = Board::new(7, 6);
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
            let mut board = Board::new(7, 6);
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
