use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct GameWrapper {
    state: mcts_connect_four::State,
}

#[wasm_bindgen]
pub enum Player {
    Player1,
    Player2,
}

#[wasm_bindgen]
impl GameWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize, cpu_is_first: bool) -> Self {
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
        Self { state }
    }

    pub fn turn(&self) -> Player {
        match self.state.turn {
            mcts_connect_four::Player::Player1 => Player::Player1,
            mcts_connect_four::Player::Player2 => Player::Player2,
        }
    }
}
