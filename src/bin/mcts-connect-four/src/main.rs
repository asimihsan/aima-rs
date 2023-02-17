use connect_four;
use monte_carlo_tree_search;

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
        todo!()
    }

    fn get_actions(&self) -> Vec<Action> {
        todo!()
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
                next_state
            }
            Turn::Player2 => {
                next_state.turn = Turn::Player1;
                next_state
            }
        }
    }
}

fn main() {
    println!("Hello, world!");
}
