# Payment Engine
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

## Unsafe stuff
- incomplete featured enabled with a flag `#![feature(type_changing_struct_update)]` ([https://rust-lang.github.io/rfcs/2528-type-changing-struct-update-syntax.html](RFC2528)) enabled me to use  syntax for destructuring structs between each other in typestate context (ok to use in this toy project)

## Missed out on some things 
- 100% test-coverage
- deep analysis and correct implementation of all business use cases and edge cases
- code comments, and CI/CD'ing `cargo doc` output into GitHub Pages with GitHub Actions
- beautiful error logs
