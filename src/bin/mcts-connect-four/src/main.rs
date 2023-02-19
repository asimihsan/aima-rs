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

#[global_allocator]
static ALLOC: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;

use rand::seq::SliceRandom;
use rand::SeedableRng;
use serde::Serialize;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Action(connect_four::Move);

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            connect_four::Move::Insert(col) => {
                serializer.serialize_str(&format!("Insert({})", col))
            }
            connect_four::Move::Pop(col) => serializer.serialize_str(&format!("Pop({})", col)),
        }
    }
}

impl monte_carlo_tree_search::Action for Action {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
enum Player {
    Player1,
    Player2,
}

impl Into<connect_four::Player> for Player {
    fn into(self) -> connect_four::Player {
        match self {
            Player::Player1 => connect_four::Player::Player1,
            Player::Player2 => connect_four::Player::Player2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
struct State {
    board: connect_four::Board,
    turn: Player,
    who_am_i: Player,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{}", self.board).fmt(f)?;

        let player = match self.turn {
            Player::Player1 => connect_four::Player::Player1,
            Player::Player2 => connect_four::Player::Player2,
        };
        writeln!(f)?;
        write!(f, "{}'s turn", player)
    }
}

impl State {
    fn new(width: usize, height: usize, turn: Player, who_am_i: Player) -> Self {
        Self {
            board: connect_four::Board::new(width, height),
            turn,
            who_am_i,
        }
    }
}

impl monte_carlo_tree_search::State<Action> for State {
    fn simulate(
        &self,
        playouts: monte_carlo_tree_search::Int,
        max_depth_per_playout: monte_carlo_tree_search::Int,
        rng: &mut monte_carlo_tree_search::Rng,
    ) -> Vec<monte_carlo_tree_search::SimulationResult> {
        (0..playouts)
            .into_iter()
            // .into_par_iter()
            .map(|_| playout(self.clone(), max_depth_per_playout, rng))
            .collect()
    }

    fn get_actions(&self) -> Vec<Action> {
        let player = match self.turn {
            Player::Player1 => connect_four::Player::Player1,
            Player::Player2 => connect_four::Player::Player2,
        };
        connect_four::get_legal_moves(&self.board, player)
            .into_iter()
            .map(Action)
            .collect()
    }

    fn get_next_state(&self, action: &Action) -> Self {
        let mut next_state = self.clone();
        let player = match self.turn {
            Player::Player1 => connect_four::Player::Player1,
            Player::Player2 => connect_four::Player::Player2,
        };
        match action {
            Action(connect_four::Move::Insert(col)) => {
                next_state.board.insert(*col, player).expect("Invalid move");
            }
            Action(connect_four::Move::Pop(col)) => {
                next_state.board.pop(*col, player).expect("Invalid move");
            }
        }

        match self.turn {
            Player::Player1 => {
                next_state.turn = Player::Player2;
            }
            Player::Player2 => {
                next_state.turn = Player::Player1;
            }
        }

        next_state
    }
}

fn playout(
    state: State,
    max_depth: monte_carlo_tree_search::Int,
    rng: &mut monte_carlo_tree_search::Rng,
) -> monte_carlo_tree_search::SimulationResult {
    let mut current_player: connect_four::Player = state.turn.into();
    let mut board = state.board;
    let mut depth = 0;
    while depth < max_depth {
        if connect_four::is_terminal_position(&board)
            != connect_four::TerminalPosition::IsNotTerminal
        {
            break;
        }

        let moves = connect_four::get_legal_moves(&board, current_player);
        if moves.is_empty() {
            break;
        }
        let random_move = moves.choose(rng).unwrap();
        match random_move {
            connect_four::Move::Insert(col) => {
                board.insert(*col, current_player).unwrap();
            }
            connect_four::Move::Pop(col) => {
                board.pop(*col, current_player).unwrap();
            }
        }
        depth += 1;
        current_player.other();
    }

    let who_am_i: connect_four::Player = state.who_am_i.into();
    if connect_four::is_terminal_position(&board)
        == connect_four::TerminalPosition::IsTerminalWin(who_am_i)
    {
        monte_carlo_tree_search::SimulationResult::Win
    } else {
        monte_carlo_tree_search::SimulationResult::NotWin
    }
}

fn get_best_mcts_move(state: &State, rng: Rc<RefCell<rand_pcg::Pcg64>>) -> connect_four::Move {
    let iterations = 200;
    let playouts_per_simulation = 2_000;
    let max_depth_per_playout = 10;
    let mut mcts = monte_carlo_tree_search::Mcts::<State, Action>::new(
        state.clone(),
        monte_carlo_tree_search::IterationLimitKind::Iterations(iterations),
        std::f64::consts::SQRT_2,
        playouts_per_simulation,
        max_depth_per_playout,
        rng,
    );

    let start = std::time::Instant::now();
    mcts.run();
    let elapsed = start.elapsed();
    let best_move = mcts.best_action().unwrap();

    println!("best move: {:?}", best_move);
    println!("elapsed: {:?}", elapsed);

    // get string serialization of tree and write to /tmp/mcts_tree.json
    let serialized_tree = mcts.serialize_tree();
    std::fs::write("/tmp/mcts_tree.json", serialized_tree).unwrap();

    best_move.0
}

fn main() {
    println!("starting");
    let rng = Rc::new(RefCell::new(rand_pcg::Pcg64::seed_from_u64(42)));
    let human_player = Player::Player1;
    let cpu_player = Player::Player2;
    let mut state = State::new(
        7,               /*width*/
        6,               /*height*/
        Player::Player1, /*turn*/
        cpu_player,      /*who_am_i*/
    );

    while connect_four::is_terminal_position(&state.board)
        == connect_four::TerminalPosition::IsNotTerminal
    {
        println!("{}", &state);
        let action = if state.turn == human_player {
            // read a character and a integer from stdin. the character is either i (insert) or p (pop).
            // the integer is the column. expect a final enter key. input is space delimited.
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            let mut input = input.split_whitespace();
            let action = input.next().unwrap();
            let col = input.next().unwrap().parse::<usize>().unwrap();
            match action {
                "i" => connect_four::Move::Insert(col),
                "p" => connect_four::Move::Pop(col),
                _ => panic!("invalid action"),
            }
        } else {
            get_best_mcts_move(&state, Rc::clone(&rng))
        };

        let player = match state.turn {
            Player::Player1 => connect_four::Player::Player1,
            Player::Player2 => connect_four::Player::Player2,
        };
        let mut new_board = state.board;
        match action {
            connect_four::Move::Insert(col) => {
                new_board.insert(col, player).unwrap();
            }
            connect_four::Move::Pop(col) => {
                new_board.pop(col, player).unwrap();
            }
        }
        state.board = new_board;
        state.turn = match state.turn {
            Player::Player1 => Player::Player2,
            Player::Player2 => Player::Player1,
        };
    }

    println!("{}", &state.board);
    match connect_four::is_terminal_position(&state.board) {
        connect_four::TerminalPosition::IsTerminalWin(winner) => {
            println!("winner: {:?}", winner);
        }
        connect_four::TerminalPosition::IsTerminalDraw => {
            println!("draw");
        }
        connect_four::TerminalPosition::IsNotTerminal => {
            panic!("game should be over");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    // player1 has two tokens in column 3 and column 4. it is player2's turn. check that mcts
    // returns a move to block on either column 2 or column 5, or else player1 will win.
    // use proptest to explore variety of playouts_per_simulation, max_depth_per_playout, exploration_constant.
    proptest! {
        #[test]
        fn test_mcts_block(
            playouts_per_simulation in 1..1000,
            max_depth_per_playout in 1..1000,
            exploration_constant in 0.0..10.0,
        ) {
            let rng = rand_pcg::Pcg64::seed_from_u64(42);
            let mut state = State::new(
                7,               /*width*/
                6,               /*height*/
                Player::Player2, /*turn*/
                Player::Player2, /*who_am_i*/
            );
            state.board.insert(3, connect_four::Player::Player1).unwrap();
            state.board.insert(4, connect_four::Player::Player1).unwrap();
            let rng = Rc::new(RefCell::new(rng));
            let mut mcts = monte_carlo_tree_search::Mcts::<State, Action>::new(
                state.clone(),
                monte_carlo_tree_search::IterationLimitKind::Iterations(100),
                exploration_constant,
                playouts_per_simulation,
                max_depth_per_playout,
                rng,
            );
            mcts.run();
            let best_move = mcts.best_action().unwrap();
            assert!(best_move.0 == connect_four::Move::Insert(2) || best_move.0 == connect_four::Move::Insert(5));
        }
    }
}
