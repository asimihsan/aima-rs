# Contributing to the project

Please read the [README](README.md) file for an introduction to the project.

## Goals

The goal of the project includes implementing a wide range of algorithms and
concepts from the textbook. The code is intended to be well-documented and easy
to understand, making it suitable for both students and professionals. However,
in many sections the textbook is merely a starting point for further research.
The goal of this project is not to provide state-of-the-art implementations of
the algorithms and concepts.

Libraries in this repository are intended for re-use. The goal is to provide a
library of components that can be used to build more complex systems. That said,
examples and pedagogical code are also welcome in the `bin` and `doc`
directories.

## Contribution guidelines

1. You agree to license your contributions under the same [LICENSE](LICENSE) as
the rest of the project. This is a requirement for all contributions. If you are
not comfortable with this, please do not contribute.
2. You agree to the [code of conduct](CODE_OF_CONDUCT.md). This is a requirement
for all contributions and participation in the project. If you are not
comfortable with this, please do not contribute or participate.
3. Code must reference "Artificial Intelligence: A Modern Approach" by Stuart
Russell and Peter Norvig, 2nd or 3rd edition. The code should be a direct
implementation of the algorithms and concepts presented in the textbook.
4. Code must be well-documented. Each function should have a doc comment
describing the purpose of the function and the parameters and return values.
5. Code must be well-tested. Each function should have unit tests that cover the
expected behavior of the function.
6. All major components and algorithms need at least one motivating end-to-end
example. This example should be included in the `bin` directory and should be
runnable with `cargo run --bin <example_name>`. The goal is to provide a working
example that demonstrates the use of the component.

## How to contribute

1. Fork the project.
2. Create a new branch for your changes.
3. Make your changes.
4. Run `cargo fmt` to format your code.
5. Run `cargo clippy` to check for common errors.
6. Run `cargo test` to run the unit tests.
7. Create a pull request.