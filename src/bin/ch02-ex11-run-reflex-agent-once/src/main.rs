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

use vacuum_cleaner::vacuum_world::{ReflexVacuumAgent, VacuumWorldEnvironment};
use vacuum_cleaner::Simulation;

// Chapter 2 Intelligent Agents Exercises 11 and 12.
//
// Exercise 11:
//
// Implement a performance-measuring environment simulator for the vacuum-cleaner world depicted in
// Figure 2.8 and specified on page . Your implementation should be modular so that the sensors,
// actuators, and environment characteristics (size, shape, dirt placement, etc.) can be changed
// easily. (Note: for some choices of programming language and operating system there are already
// implementations in the online code repository.)
fn main() {
    let agent = ReflexVacuumAgent::new();

    let width = 2;
    let height = 1;
    let environment = VacuumWorldEnvironment::new(height, width);

    let time_steps = 1000;
    let mut simulation = Simulation::new(environment, agent, time_steps);

    simulation.run();
    println!("score: {}", simulation.score());
}
