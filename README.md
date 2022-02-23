# Payment Engine
[<img alt="" src="https://img.shields.io/badge/docs-PaymentEngine-success?style=flat-square">](https://smallstepman.github.io/interview-task/interviewpuzzle/)
## Usage
### Run
```console
cargo run -- input_file.csv
```
### Test
```console
cargo test
```
## Dependencies
- `clap` - parse CLI input, a conciouss overkill
- `csv` - csv parsing and building
- `rust_decimal` - never handle money data with floats
- `serde` - well, serialization and deserialization
- `trycmd` - integration test cases self-documented in a markdown file: [./tests/README.md](./tests/README.md)
- `typestate` - cool macro to skip on writing bunch of boilerplate code when using typestate pattern, and also, automatically provides [https://smallstepman.github.io/interview-task/interviewpuzzle/ledger/transaction/index.html](awesome UML visualization of finite state machine it aids to produce)

## Unsafe stuff
- Nada

## Missed out on some things 
- 100% test-coverage
- deep analysis and correct implementation of all business use cases and edge cases
- code comments, and CI/CD'ing `cargo doc` output into GitHub Pages with GitHub Actions
- beautiful error logs
