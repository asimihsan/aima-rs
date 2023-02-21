<h1 align="center">
  aima-rs
</h1>

<h4 align="center">Rust implementation of AI concepts from "Artificial Intelligence: A Modern Approach" by Stuart Russell and Peter Norvig.</h4>

<p align="center">
  <a href="#goals">Goals</a> •
  <a href="#algorithms-and-concepts-implements">Algorithms and concepts implemented</a> •
  <a href="#contribution-guidelines">Contribution guidelines</a> •
  <a href="#license">License</a>
</p>

This project aims to provide Rust code implementations for the algorithms and
concepts presented in the textbook "Artificial Intelligence: A Modern Approach"
by Stuart Russell and Peter Norvig. The purpose of this project is to make the
algorithms and concepts from the textbook more accessible to developers by
providing working code that can be used as a learning tool or as a starting
point for further development.

## Algorithms and concepts implemented

### Chapter 2: Intelligent Agents

### Chapter 3: Solving Problems by Searching

### Chapter 4: Search in Complex Environments

### Chapter 5: Adversarial Search and Games

#### 5.4 - Monte Carlo Tree Search

Figure 5.11 page 163 `Monte-Carlo-Search` is implemented in the
[`lib/monte-carlo-tree-search`
crate](https://github.com/asimihsan/aima-rs/blob/main/src/lib/monte-carlo-tree-search/src/lib.rs).

An end-to-end example that plays Connect Four with a popout variant is
implemented using MCTS in
[`bin/mcts-connect-four`](https://github.com/asimihsan/aima-rs/blob/main/src/bin/mcts-connect-four/src/main.rs)
and can be run with `cargo run --profile production --bin mcts-connect-four`.

### Chapter 6: Constraint Satisfaction Problems

### Chapter XYZ

TODO: Add more chapters

## Goals

The goal of the project includes implementing a wide range of algorithms and
concepts from the textbook, including search algorithms, probabilistic
reasoning, and machine learning. The code is intended to be well-documented and
easy to understand, making it suitable for both students and professionals in
the field of Artificial Intelligence.

However, in many sections, the textbook is merely a starting point for further
research. The goal of this project is not to provide state-of-the-art
implementations of the algorithms and concepts, but rather to provide a starting
point for further development.

### Why Rust?

There are existing implementations of the algorithms and concepts from the
textbook in other languages such as Python and Java. However, Rust provides
developers with many features to help them develop AI applications, such as
being fast and allowing safer concurrency than other languages. Rust also allows
for stricter control of memory layout and usage, making it suitable for scaling
up algorithms on single machines for larger problems. Additionally, Rust code
can be easily compiled to WASM, making it more accessible to developers and
students alike via browser-based applications.

## Contribution guidelines

See the [CONTRIBUTING](CONTRIBUTING.md) file for details.

## Prior art

This project is inspired by the following projects:

- [aima-python](https://github.com/aimacode/aima-python)
- [aima-java](https://github.com/aimacode/aima-java)

## License

This project is licensed under the Affero General Public License (AGPL) version
3. See the [LICENSE](LICENSE) file for details.
