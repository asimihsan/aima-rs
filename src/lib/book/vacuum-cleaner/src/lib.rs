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

// PEAS - Performance, Environment, Action, Sensing
//
// See:
// -  Chapter 2: Intelligent Agents, page 40

use num_traits::Zero;

pub mod vacuum_world;

/// An Agent acts in a Performance, Environment, Action, Sensing (PEAS) cycle.
/// For a given Perception, the Agent will return an Action.
///
/// If the Agent wants to implement a table-driven agent, implementations can
/// store state of all previous Perceptions.
///
/// If the Agent wants to implement e.g. ReflexVacuumAgent, it does not need
/// to store any state.
///
/// Notice that the Agent is not aware of an Environment, it's only interface
/// is the Perception coming in then the Action going out.
pub trait Agent {
    type Action;
    type Percept;

    fn act(&mut self, percept: &Self::Percept) -> Self::Action;
}

/// An Environment runs a single Agent in a Performance, Environment, Action, Sensing (PEAS) cycle.
///
/// Notice that the Environment is not aware of an Agent.
pub trait Environment {
    type Action;
    type Percept;
    type Score: num_traits::NumAssign + Copy;

    fn percept(&self) -> Self::Percept;
    fn execute_action(&mut self, action: &Self::Action);

    /// Returns the score of the Environment. This is not cumulative or stateful. This is the score
    /// of the Environment at the current state.
    fn score(&self) -> Self::Score;
}

/// A Simulation runs a single Agent in multiple Performance, Environment, Action, Sensing (PEAS)
/// cycles. The Agent's score (Performance) is continually kept up to date.
///
/// The Simulation is aware of both the Environment and the single Agent. Notice that the Agent's
/// generic Action and Percept come from the Environment. The Agent still does not need to know that
/// the Environment exists, but the Agent definitely needs the Environment's Action and Percept
/// types.
pub struct Simulation<_Environment, _Agent>
where
    _Environment: Environment,
    _Agent: Agent<Action = _Environment::Action, Percept = _Environment::Percept>,
{
    environment: _Environment,
    agent: _Agent,
    time_steps: i32,
    score: _Environment::Score,
}

impl<_Environment, _Agent> Simulation<_Environment, _Agent>
where
    _Environment: Environment,
    _Agent: Agent<Action = _Environment::Action, Percept = _Environment::Percept>,
{
    pub fn new(environment: _Environment, agent: _Agent, time_steps: i32) -> Self {
        Self {
            environment,
            agent,
            time_steps,
            score: _Environment::Score::zero(),
        }
    }

    pub fn run(&mut self) {
        for _ in 0..self.time_steps {
            let percept = self.environment.percept();
            let action = self.agent.act(&percept);
            self.environment.execute_action(&action);
            self.score += self.environment.score();
        }
    }

    pub fn score(&self) -> <_Environment as Environment>::Score {
        self.score
    }
}
