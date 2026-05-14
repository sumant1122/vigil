# Contributing to Vigil

First off, thank you for considering contributing to Vigil! It's people like you that make Vigil such a great tool.

## How Can I Contribute?

### Reporting Bugs
* Check the existing issues to see if the bug has already been reported.
* If not, open a new issue. Include a clear title, a description of the problem, and steps to reproduce it.

### Suggesting Enhancements
* Open a new issue with the tag "enhancement".
* Describe the feature you'd like to see and why it would be useful.

### Pull Requests
1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests.
3. If you've changed APIs, update the documentation.
4. Ensure the test suite passes (`cargo test`).
5. Make sure your code lints (`cargo fmt` and `cargo clippy`).

## Development Setup

1. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
2. Clone the repo: `git clone https://github.com/sumant1122/vigil.git`
3. Build the project: `cargo build`
4. Run tests: `cargo test`

## Style Guide
We follow the standard Rust coding style. Please run `cargo fmt` before submitting your PR.

## License
By contributing, you agree that your contributions will be licensed under its MIT License.
