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

use mcts_connect_four::{MctsConfig, Player, State};
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;

fn main() {
    println!("starting");
    let mcts_config = MctsConfig::default();

    // ensure config.tree_dump_dir exists and is empty directory
    let _ = std::fs::remove_dir_all(&mcts_config.get_tree_dump_dir().unwrap());
    std::fs::create_dir(&mcts_config.get_tree_dump_dir().unwrap()).unwrap();

    let rng = Rc::new(RefCell::new(rand_pcg::Pcg64::seed_from_u64(42)));

    let human_player = Player::Player2;
    let cpu_player = Player::Player1;
    let mut state = State::new(
        7,               /*width*/
        6,               /*height*/
        Player::Player1, /*turn*/
        cpu_player,      /*who_am_i*/
    );

    while connect_four_logic::is_terminal_position(&state.board)
        == connect_four_logic::TerminalPosition::IsNotTerminal
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
                "i" => connect_four_logic::Move::Insert(col),
                "p" => connect_four_logic::Move::Pop(col),
                _ => panic!("invalid action"),
            }
        } else {
            mcts_connect_four::get_best_mcts_move(&state, &mcts_config, Rc::clone(&rng))
        };

        let player = match state.turn {
            Player::Player1 => connect_four_logic::Player::Player1,
            Player::Player2 => connect_four_logic::Player::Player2,
        };
        let mut new_board = state.board;
        match action {
            connect_four_logic::Move::Insert(col) => {
                new_board.insert(col, player).unwrap();
            }
            connect_four_logic::Move::Pop(col) => {
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
    match connect_four_logic::is_terminal_position(&state.board) {
        connect_four_logic::TerminalPosition::IsTerminalWin(winner) => {
            println!("winner: {:?}", winner);
        }
        connect_four_logic::TerminalPosition::IsTerminalDraw => {
            println!("draw");
        }
        connect_four_logic::TerminalPosition::IsNotTerminal => {
            panic!("game should be over");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // player1 has two tokens in column 3 and column 4. it is player2's turn. check that mcts
    // returns a move to block on either column 2 or column 5, or else player1 will win.
    #[test]
    fn test_avoid_losing() {
        let mcts_config = MctsConfig {
            iterations: 50,
            playouts_per_simulation: 30,
            max_depth_per_playout: 50,
            exploration_constant: std::f64::consts::SQRT_2,
            tree_dump_dir: None,
        };

        let rng = Rc::new(RefCell::new(rand_pcg::Pcg64::seed_from_u64(42)));
        let mut state = State::new(
            7,               /*width*/
            6,               /*height*/
            Player::Player2, /*turn*/
            Player::Player2, /*who_am_i*/
        );
        let board = &mut state.board;
        board
            .insert(3, connect_four_logic::Player::Player1)
            .unwrap();
        board
            .insert(4, connect_four_logic::Player::Player1)
            .unwrap();

        let best_move =
            mcts_connect_four::get_best_mcts_move(&state, &mcts_config, Rc::clone(&rng));

        assert!(
            best_move == connect_four_logic::Move::Insert(2)
                || best_move == connect_four_logic::Move::Insert(5),
            "best move: {:?}",
            best_move
        );
    }
}
