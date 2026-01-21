# Repository Guidelines

## Project Structure & Module Organization

The crate root `src/lib.rs` exposes modules dedicated to linear algebra over GF(2), including `src/mat.rs` for matrices, `src/vec.rs` for bit vectors, and `src/poly.rs` for polynomials. Shared storage and RNG helpers live in `src/store.rs` and `src/rng.rs`. Integration tests sit in `tests/` mirroring the module names, while worked examples and benchmark data live under `examples/`. Reference documentation for each module is kept in `docs/`; update the matching file when you extend an API.

## Build, Test, and Development Commands

Use `cargo build` for local compilation and `cargo test` to exercise the unit and integration suites. The examples can be run with `cargo run --example lu01` (or any other script in `examples/`). Optional nightly-only traits are enabled via `cargo build --features nightly` and should be validated with `cargo test --features nightly -- --nocapture` when touched. Generate documentation with `cargo doc --no-deps --open` before releasing.

## Coding Style & Naming Conventions

Rust 2024 defaults apply: four-space indentation, `snake_case` for modules and functions, `CamelCase` for types, and `SCREAMING_SNAKE_CASE` for constants. Format all patches with `cargo +nightly fmt` as configured in `rustfmt.toml`, and lint using `cargo clippy --all-targets --all-features`. Keep module-level comments focused on GF(2) semantics and prefer small helper functions over deeply nested bitwise expressions.

## Testing Guidelines

Add unit tests alongside new functions and extend the integration suites in `tests/` when you add public APIs. Name tests after the behaviour they cover (e.g., `mat_add_handles_zero`). Use deterministic seeds via `SmallRng::seed_from_u64` for randomized checks. When introducing new algorithms, capture edge cases for zero-width matrices, high-degree polynomials, and sparse vectors. Update or create sample runs in `examples/` if behaviour changes, and regenerate any derived data files.

## Commit & Pull Request Guidelines

The repository history is currently minimal; follow Conventional Commits (`feat:`, `fix:`, `refactor:`) with summaries under 72 characters. Each commit should compile, format, and test cleanly. Pull requests need a concise problem statement, a list of functional changes, updated docs or examples, and confirmation that `cargo test` (with `nightly` when relevant) succeeds. Link related Issues or Discussions and attach before/after output or matrices when it clarifies the behaviour.

## Environment Notes

Tests rely on the sibling crate `../utilities_rs`; ensure it is available or add a temporary `[patch]` entry for CI. Use the stable toolchain by default, but install the nightly toolchain to run the formatter and nightly feature checks.
