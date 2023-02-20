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

use crate::{Agent, Environment};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VacuumWorldLocation {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SquareState {
    Clean,
    Dirty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VacuumWorldAction {
    Left,
    Right,
    Up,
    Down,
    Suck,
    NoOp,
}

/// VacuumWorldLocalPercept is the Percept that the Agent receives from the Environment for just
/// a single location, e.g imagine a dirt sensor looking right down.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VaccuumWorldLocalPercept {
    pub location: VacuumWorldLocation,
    pub square_state: SquareState,
}

#[derive(Default)]
pub struct ReflexVacuumAgent {}

impl ReflexVacuumAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Agent for ReflexVacuumAgent {
    type Action = VacuumWorldAction;
    type Percept = VaccuumWorldLocalPercept;

    fn act(&mut self, percept: &Self::Percept) -> Self::Action {
        if percept.square_state == SquareState::Dirty {
            VacuumWorldAction::Suck
        } else if percept.location.x == 0 {
            VacuumWorldAction::Right
        } else {
            VacuumWorldAction::Left
        }
    }
}

pub struct VacuumWorldEnvironment {
    height: i32,
    width: i32,
    squares: HashMap<VacuumWorldLocation, SquareState>,
    agent_location: VacuumWorldLocation,
}

impl Default for VacuumWorldEnvironment {
    fn default() -> Self {
        let width = 2;
        let height = 1;
        VacuumWorldEnvironment::new(height, width)
    }
}

impl VacuumWorldEnvironment {
    pub fn new(height: i32, width: i32) -> Self {
        let mut squares = HashMap::new();
        for x in 0..width {
            for y in 0..height {
                squares.insert(VacuumWorldLocation { x, y }, SquareState::Dirty);
            }
        }
        Self {
            height,
            width,
            squares,
            agent_location: VacuumWorldLocation { x: 0, y: 0 },
        }
    }

    fn count_clean_squares(&self) -> i32 {
        self.squares
            .values()
            .filter(|&s| *s == SquareState::Clean)
            .count() as i32
    }
}

impl Environment for VacuumWorldEnvironment {
    type Action = VacuumWorldAction;
    type Percept = VaccuumWorldLocalPercept;
    type Score = i32;

    fn percept(&self) -> Self::Percept {
        VaccuumWorldLocalPercept {
            location: self.agent_location,
            square_state: *self.squares.get(&self.agent_location).unwrap(),
        }
    }

    fn execute_action(&mut self, action: &Self::Action) {
        match action {
            VacuumWorldAction::Left => {
                self.agent_location.x -= 1;
            }
            VacuumWorldAction::Right => {
                self.agent_location.x += 1;
            }
            VacuumWorldAction::Up => {
                self.agent_location.y += 1;
            }
            VacuumWorldAction::Down => {
                self.agent_location.y -= 1;
            }
            VacuumWorldAction::Suck => {
                self.squares.insert(self.agent_location, SquareState::Clean);
            }
            VacuumWorldAction::NoOp => {}
        }
        self.agent_location.x = self.agent_location.x.clamp(0, self.width - 1);
        self.agent_location.y = self.agent_location.y.clamp(0, self.height - 1);
    }

    fn score(&self) -> Self::Score {
        self.count_clean_squares()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflex_vacuum_agent() {
        let mut agent = ReflexVacuumAgent::new();
        let percept = VaccuumWorldLocalPercept {
            location: VacuumWorldLocation { x: 0, y: 0 },
            square_state: SquareState::Dirty,
        };
        assert_eq!(agent.act(&percept), VacuumWorldAction::Suck);
        let percept = VaccuumWorldLocalPercept {
            location: VacuumWorldLocation { x: 1, y: 0 },
            square_state: SquareState::Dirty,
        };
        assert_eq!(agent.act(&percept), VacuumWorldAction::Suck);
        let percept = VaccuumWorldLocalPercept {
            location: VacuumWorldLocation { x: 0, y: 0 },
            square_state: SquareState::Clean,
        };
        assert_eq!(agent.act(&percept), VacuumWorldAction::Right);
        let percept = VaccuumWorldLocalPercept {
            location: VacuumWorldLocation { x: 1, y: 0 },
            square_state: SquareState::Clean,
        };
        assert_eq!(agent.act(&percept), VacuumWorldAction::Left);
    }

    #[test]
    fn test_vacuum_world_environment_returns_dirty_percept() {
        let env = VacuumWorldEnvironment::default();
        let percept = env.percept();
        assert_eq!(percept.location, VacuumWorldLocation { x: 0, y: 0 });
        assert_eq!(percept.square_state, SquareState::Dirty);
    }

    #[test]
    fn test_vacuum_world_environment_allows_cleaning() {
        let mut env = VacuumWorldEnvironment::default();
        env.execute_action(&VacuumWorldAction::Suck);
        let percept = env.percept();
        assert_eq!(percept.location, VacuumWorldLocation { x: 0, y: 0 });
        assert_eq!(percept.square_state, SquareState::Clean);
    }

    #[test]
    fn test_vacuum_world_environment_cleaning_leaves_other_square_dirty() {
        let mut env = VacuumWorldEnvironment::default();
        env.execute_action(&VacuumWorldAction::Suck);
        let percept = env.percept();
        assert_eq!(percept.location, VacuumWorldLocation { x: 0, y: 0 });
        assert_eq!(percept.square_state, SquareState::Clean);
        env.execute_action(&VacuumWorldAction::Right);
        let percept = env.percept();
        assert_eq!(percept.location, VacuumWorldLocation { x: 1, y: 0 });
        assert_eq!(percept.square_state, SquareState::Dirty);
    }

    #[test]
    fn test_vacuum_world_environment_score() {
        let mut env = VacuumWorldEnvironment::default();
        assert_eq!(env.score(), 0);
        env.execute_action(&VacuumWorldAction::Suck);
        assert_eq!(env.score(), 1);
        env.execute_action(&VacuumWorldAction::Right);
        assert_eq!(env.score(), 1);
        env.execute_action(&VacuumWorldAction::Suck);
        assert_eq!(env.score(), 2);
    }

    #[test]
    fn test_vacuum_world_noop_does_nothing() {
        let mut env = VacuumWorldEnvironment::default();
        env.execute_action(&VacuumWorldAction::NoOp);
        let percept = env.percept();
        assert_eq!(percept.location, VacuumWorldLocation { x: 0, y: 0 });
        assert_eq!(percept.square_state, SquareState::Dirty);
    }
}
