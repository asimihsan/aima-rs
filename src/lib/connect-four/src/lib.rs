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

    pub fn get(&self, col: usize, row: usize) -> Option<Cell> {
        if col < WIDTH && row < HEIGHT {
            Some(self.cells[row][col])
        } else {
            None
        }
    }

    // insert will insert a piece into the board. It will return None if the column is full.
    // This inserts into the first empty cell in the column, going from the bottom up.
    pub fn insert(&mut self, col: usize, player: Player) -> Option<()> {
        if col >= WIDTH {
            return None;
        }
        for row in (0..HEIGHT).rev() {
            if self.cells[row][col] == Cell::Empty {
                self.cells[row][col] = match player {
                    Player::Player1 => Cell::Player1,
                    Player::Player2 => Cell::Player2,
                };
                return Some(());
            }
        }
        None
    }

    // pop will remove a piece from the board. It will return None if the column is empty.
    // This removes the first non-empty cell in the column, going from the bottom up.
    // This will shift down all the pieces above it. This is used for the popout variant.
    pub fn pop(&mut self, col: usize) -> Option<()> {
        if col >= WIDTH {
            return None;
        }
        for row in (0..HEIGHT).rev() {
            if self.cells[row][col] != Cell::Empty {
                self.cells[row][col] = Cell::Empty;
                for row in row..HEIGHT - 1 {
                    self.cells[row][col] = self.cells[row + 1][col];
                }
                return Some(());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_starts_empty() {
        let mut board = Board::<7, 6>::new();
        for row in 0..6 {
            for col in 0..7 {
                assert_eq!(
                    board.get(col, row),
                    Some(Cell::Empty),
                    "col: {}, row: {}",
                    col,
                    row
                );
            }
        }
    }

    #[test]
    fn test_board_one_insert() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1);

        for row in 0..6 {
            for col in 0..7 {
                if col == 0 && row == 5 {
                    assert_eq!(board.get(col, row), Some(Cell::Player1));
                } else {
                    assert_eq!(
                        board.get(col, row),
                        Some(Cell::Empty),
                        "col: {}, row: {}",
                        col,
                        row
                    );
                }
            }
        }
    }

    #[test]
    fn test_board_one_insert_then_pop() {
        let mut board = Board::<7, 6>::new();
        board.insert(0, Player::Player1);
        board.pop(0);

        for row in 0..6 {
            for col in 0..7 {
                assert_eq!(
                    board.get(col, row),
                    Some(Cell::Empty),
                    "col: {}, row: {}",
                    col,
                    row
                );
            }
        }
    }
}
