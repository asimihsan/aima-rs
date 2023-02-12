/*
 * Copyright 2023 Asim Ihsan
 * SPDX-License-Identifier: Apache-2.0
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
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
