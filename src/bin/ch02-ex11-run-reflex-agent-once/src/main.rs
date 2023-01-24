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
