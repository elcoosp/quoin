
# Contributing to quoin

Thank you for your interest in contributing! We welcome all contributions—code, docs, examples, and ideas.

## Getting Started

1. Fork the repository and clone it locally.
2. Install Rust (stable) and run `cargo build` to verify everything compiles.
3. Make your changes on a feature branch.

## Development Workflow

- Run tests: `cargo test --all`
- Check formatting: `cargo fmt --all -- --check`
- Run lints: `cargo clippy --all -- -D warnings`
- Build docs: `cargo doc --no-deps`

## Adding a New Adapter

1. Create a new crate `quoin-<framework>` in the workspace.
2. Implement `quoin::ReactiveContext` for your framework`s reactive context.
3. Add a conformance test using `quoin-conformance` and `tested-trait`.
4. Add a README following the pattern of existing adapters.
5. Open a PR!

## Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## License

By contributing, you agree that your contributions will be dual-licensed under MIT and Apache-2.0.

