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
use std::rc::Rc;

use rand_core::SeedableRng;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameWrapper {
    state: mcts_connect_four::State,
    mcts_config: mcts_connect_four::MctsConfig,
    rng: Rc<RefCell<rand_pcg::Pcg64>>,
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
        let mut result = String::new();
        match self.state.turn {
            mcts_connect_four::Player::Player1 => {
                result.push_str("Player 1");
            }
            mcts_connect_four::Player::Player2 => {
                result.push_str("Player 2");
            }
        }
        if self.state.turn == self.state.who_am_i {
            result.push_str(" (CPU)");
        } else {
            result.push_str(" (human)");
        }

        serde_wasm_bindgen::to_value(&result).unwrap()
    }

    pub fn width(&self) -> usize {
        self.state.board.width
    }

    pub fn height(&self) -> usize {
        self.state.board.height
    }

    pub fn get_mcts_best_move(&self) -> JsValue {
        let action = mcts_connect_four::get_best_mcts_move(
            &self.state,
            &self.mcts_config,
            Rc::clone(&self.rng),
        );
        let result = serde_json::to_string(&action).unwrap();
        JsValue::from_str(&result)
    }
}
