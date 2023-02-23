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

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use rand::prelude::SliceRandom;
use serde::ser::Serialize;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Action(connect_four_logic::Move);

impl Serialize for Action {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self.0 {
            connect_four_logic::Move::Insert(col) => {
                serializer.serialize_str(&format!("Insert({})", col))
            }
            connect_four_logic::Move::Pop(col) => {
                serializer.serialize_str(&format!("Pop({})", col))
            }
        }
    }
}

impl monte_carlo_tree_search::Action for Action {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Player {
    Player1,
    Player2,
}

impl From<Player> for connect_four_logic::Player {
    fn from(player: Player) -> Self {
        match player {
            Player::Player1 => connect_four_logic::Player::Player1,
            Player::Player2 => connect_four_logic::Player::Player2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct State {
    pub board: connect_four_logic::Board,
    pub turn: Player,
    pub who_am_i: Player,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{}", self.board).fmt(f)?;

        let player = match self.turn {
            Player::Player1 => connect_four_logic::Player::Player1,
            Player::Player2 => connect_four_logic::Player::Player2,
        };
        writeln!(f)?;
        write!(f, "{}'s turn", player)
    }
}

impl State {
    pub fn new(width: usize, height: usize, turn: Player, who_am_i: Player) -> Self {
        Self {
            board: connect_four_logic::Board::new(width, height),
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
            Player::Player1 => connect_four_logic::Player::Player1,
            Player::Player2 => connect_four_logic::Player::Player2,
        };
        connect_four_logic::get_legal_moves(&self.board, player)
            .into_iter()
            .map(Action)
            .collect()
    }

    fn get_next_state(&self, action: &Action) -> Self {
        let mut next_state = self.clone();
        let player = match self.turn {
            Player::Player1 => connect_four_logic::Player::Player1,
            Player::Player2 => connect_four_logic::Player::Player2,
        };
        match action {
            Action(connect_four_logic::Move::Insert(col)) => {
                next_state.board.insert(*col, player).expect("Invalid move");
            }
            Action(connect_four_logic::Move::Pop(col)) => {
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

    fn is_terminal(&self) -> bool {
        connect_four_logic::is_terminal_position(&self.board)
            != connect_four_logic::TerminalPosition::IsNotTerminal
    }
}

fn playout(
    state: State,
    max_depth: monte_carlo_tree_search::Int,
    rng: &mut monte_carlo_tree_search::Rng,
) -> monte_carlo_tree_search::SimulationResult {
    let mut current_player: connect_four_logic::Player = state.turn.into();
    let mut board = state.board;
    let mut depth = 0;
    while depth < max_depth {
        if connect_four_logic::is_terminal_position(&board)
            != connect_four_logic::TerminalPosition::IsNotTerminal
        {
            break;
        }

        let moves = connect_four_logic::get_legal_moves(&board, current_player);
        if moves.is_empty() {
            break;
        }

        // Check if any of the moves are winning moves. If so, take that move.
        let mut used_winning_move = false;
        for m in moves.iter() {
            let mut board_copy = board.clone();
            match m {
                connect_four_logic::Move::Insert(col) => {
                    board_copy.insert(*col, current_player).unwrap();
                }
                connect_four_logic::Move::Pop(col) => {
                    board_copy.pop(*col, current_player).unwrap();
                }
            }
            if connect_four_logic::is_terminal_position(&board_copy)
                == connect_four_logic::TerminalPosition::IsTerminalWin(current_player)
            {
                used_winning_move = true;
                board = board_copy;
                depth += 1;
                current_player.other();
                break;
            }
        }
        if used_winning_move {
            break;
        }

        let random_move = moves.choose(rng).unwrap();
        match random_move {
            connect_four_logic::Move::Insert(col) => {
                board.insert(*col, current_player).unwrap();
            }
            connect_four_logic::Move::Pop(col) => {
                board.pop(*col, current_player).unwrap();
            }
        }
        depth += 1;
        current_player.other();
    }

    let who_am_i: connect_four_logic::Player = state.who_am_i.into();
    if connect_four_logic::is_terminal_position(&board)
        == connect_four_logic::TerminalPosition::IsTerminalWin(who_am_i)
    {
        monte_carlo_tree_search::SimulationResult::Win
    } else {
        monte_carlo_tree_search::SimulationResult::NotWin
    }
}

pub struct MctsConfig {
    pub iterations: monte_carlo_tree_search::Int,
    pub exploration_constant: monte_carlo_tree_search::Float,
    pub playouts_per_simulation: monte_carlo_tree_search::Int,
    pub max_depth_per_playout: monte_carlo_tree_search::Int,
    pub tree_dump_dir: Option<PathBuf>,
}

impl MctsConfig {
    fn new(
        iterations: monte_carlo_tree_search::Int,
        exploration_constant: monte_carlo_tree_search::Float,
        playouts_per_simulation: monte_carlo_tree_search::Int,
        max_depth_per_playout: monte_carlo_tree_search::Int,
        tree_dump_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            iterations,
            exploration_constant,
            playouts_per_simulation,
            max_depth_per_playout,
            tree_dump_dir,
        }
    }

    pub fn get_tree_dump_dir(&self) -> Option<PathBuf> {
        self.tree_dump_dir.clone()
    }
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self::new(
            300,
            std::f64::consts::SQRT_2,
            500,
            50,
            Some(PathBuf::from("/tmp/tree-dump-dir")),
        )
    }
}

pub fn get_best_mcts_move(
    state: &State,
    config: &MctsConfig,
    rng: Rc<RefCell<rand_pcg::Pcg64>>,
) -> connect_four_logic::Move {
    let mut mcts = monte_carlo_tree_search::Mcts::<State, Action>::new(
        state.clone(),
        monte_carlo_tree_search::IterationLimitKind::Iterations(config.iterations),
        config.exploration_constant,
        config.playouts_per_simulation,
        config.max_depth_per_playout,
        rng,
        config.tree_dump_dir.clone(),
    );

    mcts.run();
    let best_move = mcts.best_action().unwrap();
    println!("best move: {:?}", best_move);

    best_move.0
}
