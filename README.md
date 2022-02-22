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
- `trycmd` - integration test cases self-documented in a markdown file

## Other
I've figured you're most likely intersted in seeing whether I can write idiomatic, correct and performant Rust, while keeping tidy Git log. With that idea in mind, I missed out on some things (either purposely, or due to lack of time):
- 100% test-coverage
- deep analysis and correct implementation of all business use cases and edge cases
- typestate pattern
- comments, and CI/CD'ing `cargo doc` output into GitHub Pages using GitHub Actions
- creating issues, branches, PRs
