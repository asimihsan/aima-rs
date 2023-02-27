/*
 * Copyright (C) 2023 Asim Ihsan
 * SPDX-License-Identifier: AGPL-3.0-only
 *
 * This program is free software: you can redistribute it and/or modify it under
 * the terms of the GNU Affero General Public License as published by the Free
 * Software Foundation, version 3.
 *
 * This program is distributed in the hope that it will be useful, but WITHOUT ANY
 * WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
 * PARTICULAR PURPOSE. See the GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License along
 * with this program. If not, see <https://www.gnu.org/licenses/>
 */

#![warn(missing_docs)]

//! Connect Four game logic.
//!
//! This is a library for the Connect Four game. It is intended to be used by
//! an algorithm to simulate or play the game.

use serde::{Deserialize, Serialize};

/// Connect Four error.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ConnectFourError {
    /// Column is full.
    #[error("column is full: {0}")]
    ColumnFull(usize),

    /// Column is empty.
    #[error("column is empty: {0}")]
    ColumnEmpty(usize),

    /// Column is not yours. You can only pop from your own columns.
    #[error("column is not yours: {0}")]
    ColumnNotYours(usize),
}

/// Connect Four cell. Part of the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cell {
    /// Empty cell.
    Empty,

    /// Cell for a piece belonging to a player.
    Player(Player),
}

/// Connect Four player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    /// Player 1.
    Player1,

    /// Player 2.
    Player2,
}

impl Player {
    /// Get the other player.
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

/// Connect Four board. This only contains the cells, and not the players or the turn.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Board {
    /// Cells, either empty or containing a player.
    pub cells: Vec<Cell>,

    /// Width of the board.
    pub width: usize,

    /// Height of the board.
    pub height: usize,
}

// print out cells, and row and column numbers which start at 0.
impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::with_capacity((self.width * 2 + 1) * (self.height + 1));

        // print column numbers. recall there will be row numbers on the left.
        for col in 0..self.width {
            if col == 0 {
                s.push_str("  ");
            }

            s.push_str(&format!("{}", col));
            if col == self.width - 1 {
                s.push('\n');
            } else {
                s.push(' ');
            }
        }

        for row in 0..self.height {
            // print row numbers
            s.push_str(&format!("{} ", row));

            for col in 0..self.width {
                let cell = self.get(col, row);
                let c = match cell {
                    Cell::Empty => '.',
                    Cell::Player(Player::Player1) => '1',
                    Cell::Player(Player::Player2) => '2',
                };
                s.push(c);
                if col < self.width - 1 {
                    s.push(' ');
                }
            }
            if row < self.height - 1 {
                s.push('\n');
            }
        }
        write!(f, "{}", s)
    }
}

impl Board {
    /// Create a new board.
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::Empty; width * height];
        Self {
            cells,
            width,
            height,
        }
    }

    /// Get a cell.
    pub fn get(&self, col: usize, row: usize) -> Cell {
        self.cells[row * self.width + col]
    }

    /// Get a mutable cell.
    pub fn get_mut(&mut self, col: usize, row: usize) -> &mut Cell {
        &mut self.cells[row * self.width + col]
    }

    /// Get a column of cells.
    pub fn get_col(&self, col: usize) -> Vec<Cell> {
        (0..self.height).map(|row| self.get(col, row)).collect()
    }

    /// Check if you can insert a piece into a column. Return the row where the inserted piece will
    /// be.
    pub fn can_insert(&self, col: usize) -> Result<usize, ConnectFourError> {
        for row in (0..self.height).rev() {
            if self.get(col, row) == Cell::Empty {
                return Ok(row);
            }
        }
        Err(ConnectFourError::ColumnFull(col))
    }

    /// insert will insert a piece into the board. It will return None if the column is full.
    /// This inserts into the first empty cell in the column, going from the bottom up.
    pub fn insert(&mut self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        match self.can_insert(col) {
            Ok(row) => {
                let cell = self.get_mut(col, row);
                *cell = Cell::Player(player);
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    /// Check if you can pop a piece from a column.
    pub fn can_pop(&self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        let col_cells = self.get_col(col);
        let last_cell = col_cells[self.height - 1];
        match last_cell {
            Cell::Empty => return Err(ConnectFourError::ColumnEmpty(col)),
            Cell::Player(p) => {
                if p != player {
                    return Err(ConnectFourError::ColumnNotYours(col));
                }
            }
        }
        Ok(())
    }

    /// pop will remove a piece from the board. It will return None if the column is empty.
    /// This removes the first non-empty cell in the column, going from the bottom up.
    /// This will shift down all the pieces above it. This is used for the popout variant.
    ///
    /// You can only pop from a column if the bottom piece is yours.
    pub fn pop(&mut self, col: usize, player: Player) -> Result<(), ConnectFourError> {
        match self.can_pop(col, player) {
            Ok(()) => {
                // for this column, copy the i+1 higher element down to i, in reverse order
                for row in (0..self.height - 1).rev() {
                    let cell1 = self.get(col, row);
                    let cell2 = self.get_mut(col, row + 1);
                    *cell2 = cell1;
                }

                // set the top cell to empty
                let cell = self.get_mut(col, 0 /*row*/);
                *cell = Cell::Empty;

                Ok(())
            }
            Err(e) => Err(e),
        }
    }
}

/// MoveType is either Insert or Pop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveType {
    /// Insert a piece into a column.
    Insert,

    /// Pop a piece from a column.
    Pop,
}

/// Move is a move type and a column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    /// The move type.
    pub move_type: MoveType,

    /// The column.
    pub column: usize,
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Move {
                move_type: MoveType::Insert,
                column: col,
            } => write!(f, "Insert({})", col),
            Move {
                move_type: MoveType::Pop,
                column: col,
            } => write!(f, "Pop({})", col),
        }
    }
}

/// Get all the legal moves for a player.
pub fn get_legal_moves(board: &Board, player: Player) -> Vec<Move> {
    let mut moves = Vec::new();
    for col in 0..board.width {
        if board.can_insert(col).is_ok() {
            moves.push(Move {
                move_type: MoveType::Insert,
                column: col,
            });
        }
        if board.can_pop(col, player).is_ok() {
            moves.push(Move {
                move_type: MoveType::Pop,
                column: col,
            });
        }
    }
    moves
}

/// Whether a position is terminal, and if so, who won.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalPosition {
    /// The game is terminal, and some Player has won.
    IsTerminalWin(Player),

    /// The game is terminal, and it is a draw.
    IsTerminalDraw,

    /// The game is not terminal.
    IsNotTerminal,
}

/// Check if a position is terminal.
pub fn is_terminal_position(board: &Board) -> TerminalPosition {
    // check for a win
    for row in 0..board.height {
        for col in 0..board.width {
            let cell1 = board.get(col, row);
            if cell1 == Cell::Empty {
                continue;
            }
            let player = match cell1 {
                Cell::Player(p) => p,
                Cell::Empty => unreachable!(),
            };

            // check horizontal
            if col + 3 < board.width {
                let cell2 = board.get(col + 1, row);
                let cell3 = board.get(col + 2, row);
                let cell4 = board.get(col + 3, row);
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check vertical
            if row + 3 < board.height {
                let cell2 = board.get(col, row + 1);
                let cell3 = board.get(col, row + 2);
                let cell4 = board.get(col, row + 3);
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check diagonal down
            if col + 3 < board.width && row + 3 < board.height {
                let cell2 = board.get(col + 1, row + 1);
                let cell3 = board.get(col + 2, row + 2);
                let cell4 = board.get(col + 3, row + 3);
                if cell1 == cell2 && cell2 == cell3 && cell3 == cell4 {
                    return TerminalPosition::IsTerminalWin(player);
                }
            }

            // check diagonal up
            if col + 3 < board.width && row >= 3 {
                let cell2 = board.get(col + 1, row - 1);
                let cell3 = board.get(col + 2, row - 2);
                let cell4 = board.get(col + 3, row - 3);
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
    use proptest::prelude::*;

    use super::*;

    fn all_cells_empty(board: &Board) -> bool {
        for row in 0..board.height {
            for col in 0..board.width {
                if board.get(col, row) != Cell::Empty {
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
                let get = board.get(col, row);
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
                assert!(legal_moves.contains(&Move {
                    move_type: MoveType::Insert,
                    column: col,
                }));
                assert!(!legal_moves.contains(&Move {
                    move_type: MoveType::Pop,
                    column: col,
                }));
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
                let get = board.get(col, row);
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
        let col0 = board.get_col(0);
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
        let col0 = board.get_col(0);
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
        let col0 = board.get_col(0);
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

    // when a column is full, then when you pop the column must not be full.
    #[test]
    fn test_column_full_then_pop_means_not_full() {
        let mut board = Board::new(7, 6);
        for _ in 0..6 {
            board.insert(0, Player::Player1).expect("insert failed");
        }
        let col0_before = board.get_col(0);
        assert!(col0_before
            .iter()
            .all(|&cell| cell == Cell::Player(Player::Player1)));
        assert_eq!(Err(ConnectFourError::ColumnFull(0)), board.can_insert(0));

        board.pop(0, Player::Player1).expect("pop failed");

        let col0_after = board.get_col(0);
        assert_eq!(Cell::Empty, col0_after[0]);
        assert_eq!(Ok(0), board.can_insert(0));
    }

    fn vec_of_player() -> impl Strategy<Value = Vec<Player>> {
        prop::collection::vec(
            prop_oneof![Just(Player::Player1), Just(Player::Player2)],
            1..6,
        )
    }

    proptest! {
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

            let initial_col = board.get_col(col);

            let first_player = &player.first().unwrap();
            board.pop(col, **first_player).expect("pop failed");
            let final_col = board.get_col(col);

            let initial_col_non_empty = initial_col.iter().filter(|c| **c != Cell::Empty).collect::<Vec<_>>();
            let final_col_non_empty = final_col.iter().filter(|c| **c != Cell::Empty).collect::<Vec<_>>();

            // final_col should have one more empty cell that initial_col
            assert_eq!(initial_col_non_empty.len(), final_col_non_empty.len() + 1);

            // initial_col_non_empty except last element is same as final
            assert_eq!(initial_col_non_empty[..initial_col_non_empty.len() - 1], final_col_non_empty[..]);
        }
    }
}
