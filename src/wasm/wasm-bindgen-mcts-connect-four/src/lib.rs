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
use std::collections::BTreeSet;
use std::rc::Rc;

use rand_core::SeedableRng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use monte_carlo_tree_search::State;

#[wasm_bindgen]
pub struct GameWrapper {
    state: mcts_connect_four::State,
    mcts_config: mcts_connect_four::MctsConfig,
    rng: Rc<RefCell<rand_pcg::Pcg64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Turn {
    Player1,
    Player2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MoveType {
    Insert,
    Pop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Move {
    pub move_type: MoveType,
    pub column: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoveResponse {
    pub actual_move: Move,
    pub maybe_insert_row: Option<usize>,
    pub debug_trees: Vec<
        monte_carlo_tree_search::MctsNodeForSerialization<
            mcts_connect_four::State,
            mcts_connect_four::Action,
        >,
    >,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ApplyMoveRequest {
    pub move_type: MoveType,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LegalMove {
    pub move_type: MoveType,
    pub row: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LegalMoveResponse {
    pub moves: Vec<LegalMove>,
}

#[wasm_bindgen]
impl GameWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, cpu_is_first: bool) -> Self {
        console_error_panic_hook::set_once();

        let (turn, who_am_i) = if cpu_is_first {
            (
                mcts_connect_four::Player::Player1,
                mcts_connect_four::Player::Player1,
            )
        } else {
            (
                mcts_connect_four::Player::Player1,
                mcts_connect_four::Player::Player2,
            )
        };

        let state = mcts_connect_four::State::new(width, height, turn, who_am_i);
        let mcts_config = mcts_connect_four::MctsConfig {
            tree_dump_dir: None,
            debug_track_trees: monte_carlo_tree_search::DebugTrackTrees::Track,
            ..mcts_connect_four::MctsConfig::default()
        };
        let rng = Rc::new(RefCell::new(rand_pcg::Pcg64::seed_from_u64(42)));
        Self {
            state,
            mcts_config,
            rng,
        }
    }

    /// turn is the player whose turn it is to make a move and whether they are a human or a CPU.
    /// e.g. "Player 1 (human).
    pub fn turn(&self) -> JsValue {
        let turn = match self.state.turn {
            mcts_connect_four::Player::Player1 => Turn::Player1,
            mcts_connect_four::Player::Player2 => Turn::Player2,
        };
        serde_wasm_bindgen::to_value(&turn).unwrap()
    }

    pub fn width(&self) -> usize {
        self.state.board.width
    }

    pub fn height(&self) -> usize {
        self.state.board.height
    }

    pub fn get_mcts_best_move(&self) -> Result<JsValue, JsValue> {
        if self.state.turn != self.state.who_am_i {
            return Err(serde_wasm_bindgen::to_value("Not CPU's turn").unwrap());
        }

        let action = mcts_connect_four::get_best_mcts_move(
            &self.state,
            &self.mcts_config,
            Rc::clone(&self.rng),
        );

        // If this is an insert, then use can_insert to get the row.
        let response = if action.actual_move.move_type == connect_four_logic::MoveType::Insert {
            let maybe_insert_row = self
                .state
                .board
                .can_insert(action.actual_move.column)
                .unwrap();
            MoveResponse {
                actual_move: Move {
                    move_type: MoveType::Insert,
                    column: action.actual_move.column,
                },
                maybe_insert_row: Some(maybe_insert_row),
                debug_trees: action.debug_trees.unwrap(),
            }
        } else {
            MoveResponse {
                actual_move: Move {
                    move_type: MoveType::Pop,
                    column: action.actual_move.column,
                },
                maybe_insert_row: None,
                debug_trees: action.debug_trees.unwrap(),
            }
        };
        Ok(serde_wasm_bindgen::to_value(&response).unwrap())
    }

    pub fn apply_move(&mut self, apply_move_request: JsValue) -> Result<JsValue, JsValue> {
        let apply_move_request: ApplyMoveRequest =
            serde_wasm_bindgen::from_value(apply_move_request)?;
        let action = match apply_move_request.move_type {
            MoveType::Insert => connect_four_logic::MoveType::Insert,
            MoveType::Pop => connect_four_logic::MoveType::Pop,
        };
        let action = mcts_connect_four::Action(connect_four_logic::Move {
            move_type: action,
            column: apply_move_request.column,
        });
        self.state.apply_move(&action);
        let result = serde_wasm_bindgen::to_value(&self.state).unwrap();
        Ok(result)
    }

    /// get_legal_moves_cells will return cells on which the current player can move. For
    /// an insert the cell will be the first empty cell top-down in a column. For a pop it will
    /// be the bottom of the column. self.state.get_actions returns moves, but we need to return
    /// LegalMoveResponse, which has Vec<LegalMove>
    pub fn get_legal_moves_cells(&self) -> JsValue {
        let legal_moves = self.state.get_actions();
        let moves: Vec<LegalMove> = legal_moves
            .into_iter()
            .map(|action| {
                let actual_move = action.0;
                let column = actual_move.column;
                match actual_move.move_type {
                    connect_four_logic::MoveType::Insert => {
                        let row = self.state.board.can_insert(actual_move.column).unwrap();
                        LegalMove {
                            move_type: MoveType::Insert,
                            row,
                            column,
                        }
                    }
                    connect_four_logic::MoveType::Pop => LegalMove {
                        move_type: MoveType::Pop,
                        column,
                        row: self.height() - 1,
                    },
                }
            })
            .collect();
        let result = LegalMoveResponse { moves };
        serde_wasm_bindgen::to_value(&result).unwrap()
    }
}

#[wasm_bindgen]
pub struct ClickDebouncer {
    clicks: BTreeSet<(usize, usize)>,
}

#[wasm_bindgen]
impl ClickDebouncer {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            clicks: BTreeSet::new(),
        }
    }

    pub fn add(&mut self, row: usize, col: usize) {
        self.clicks.insert((row, col));
    }

    pub fn is_present(&self, row: usize, col: usize) -> bool {
        self.clicks.contains(&(row, col))
    }

    pub fn clear(&mut self) {
        self.clicks.clear();
    }
}
