use rand::seq::SliceRandom;
use rand::SeedableRng;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Action(connect_four::Move);

impl monte_carlo_tree_search::Action for Action {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Turn {
    Player1,
    Player2,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct State<const WIDTH: usize, const HEIGHT: usize> {
    board: connect_four::Board<WIDTH, HEIGHT>,
    turn: Turn,
}

impl<const WIDTH: usize, const HEIGHT: usize> std::fmt::Display for State<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format!("{}", self.board).fmt(f)?;

        let player = match self.turn {
            Turn::Player1 => connect_four::Player::Player1,
            Turn::Player2 => connect_four::Player::Player2,
        };
        writeln!(f)?;
        write!(f, "{}'s turn", player)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> State<WIDTH, HEIGHT> {
    fn new() -> Self {
        Self {
            board: connect_four::Board::new(),
            turn: Turn::Player1,
        }
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> monte_carlo_tree_search::State<Action>
    for State<WIDTH, HEIGHT>
{
    fn simulate(
        &self,
        playouts: monte_carlo_tree_search::Int,
        max_depth_per_playout: monte_carlo_tree_search::Int,
        rng: &mut monte_carlo_tree_search::Rng,
    ) -> Vec<monte_carlo_tree_search::SimulationResult> {
        (0..playouts)
            .map(|_| playout(self.clone(), max_depth_per_playout, rng))
            .collect()
    }

    fn get_actions(&self) -> Vec<Action> {
        let player = match self.turn {
            Turn::Player1 => connect_four::Player::Player1,
            Turn::Player2 => connect_four::Player::Player2,
        };
        connect_four::get_legal_moves(&self.board, player)
            .into_iter()
            .map(Action)
            .collect()
    }

    fn get_next_state(&self, action: &Action) -> Self {
        let mut next_state = self.clone();
        let player = match self.turn {
            Turn::Player1 => connect_four::Player::Player1,
            Turn::Player2 => connect_four::Player::Player2,
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
            Turn::Player1 => {
                next_state.turn = Turn::Player2;
            }
            Turn::Player2 => {
                next_state.turn = Turn::Player1;
            }
        }

        next_state
    }
}

fn playout<const WIDTH: usize, const HEIGHT: usize>(
    state: State<WIDTH, HEIGHT>,
    max_depth: monte_carlo_tree_search::Int,
    rng: &mut monte_carlo_tree_search::Rng,
) -> monte_carlo_tree_search::SimulationResult {
    let mut player = match state.turn {
        Turn::Player1 => connect_four::Player::Player1,
        Turn::Player2 => connect_four::Player::Player2,
    };
    let mut board = state.board;
    let mut depth = 0;
    while depth < max_depth {
        if connect_four::is_terminal_position(&board)
            != connect_four::TerminalPosition::IsNotTerminal
        {
            break;
        }

        let moves = connect_four::get_legal_moves(&board, player);
        if moves.is_empty() {
            break;
        }
        let random_move = moves.choose(rng).unwrap();
        match random_move {
            connect_four::Move::Insert(col) => {
                board.insert(*col, player).unwrap();
            }
            connect_four::Move::Pop(col) => {
                board.pop(*col, player).unwrap();
            }
        }
        depth += 1;
        player = match player {
            connect_four::Player::Player1 => connect_four::Player::Player2,
            connect_four::Player::Player2 => connect_four::Player::Player1,
        };
    }
    match connect_four::is_terminal_position(&board) {
        connect_four::TerminalPosition::IsTerminalWin(winner) => {
            if winner == player {
                monte_carlo_tree_search::SimulationResult::Win
            } else {
                monte_carlo_tree_search::SimulationResult::NotWin
            }
        }
        _ => monte_carlo_tree_search::SimulationResult::NotWin,
    }
}

fn get_best_mcts_move<const WIDTH: usize, const HEIGHT: usize>(
    state: &State<WIDTH, HEIGHT>,
    rng: Rc<RefCell<rand_pcg::Pcg64>>,
) -> connect_four::Move {
    let iterations = 100;
    let playouts_per_simulation = 100;
    let max_depth_per_playout = 20;
    let mut mcts = monte_carlo_tree_search::Mcts::<State<WIDTH, HEIGHT>, Action>::new(
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

    best_move.0
}

fn main() {
    println!("starting");
    let mut rng = Rc::new(RefCell::new(rand_pcg::Pcg64::seed_from_u64(42)));
    let human_player = connect_four::Player::Player1;
    let mut state = State::<7, 6>::new();

    while connect_four::is_terminal_position(&state.board)
        == connect_four::TerminalPosition::IsNotTerminal
    {
        println!("{}", &state);
        let player = match state.turn {
            Turn::Player1 => connect_four::Player::Player1,
            Turn::Player2 => connect_four::Player::Player2,
        };
        let action = if player == human_player {
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
            Turn::Player1 => Turn::Player2,
            Turn::Player2 => Turn::Player1,
        };
    }

    println!("{}", &state);
}
