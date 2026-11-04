# Contributing to quoin

Thank you for your interest in contributing! We welcome all contributions—code, docs, examples, and ideas.

## Getting Started

1. Fork the repository and clone it locally.
2. Install Rust stable (**1.88+**, enforced via `rust-version`) and run `cargo build` to verify everything compiles.
3. Install the recommended tooling:
   - [`just`](https://github.com/casey/just) — task runner used by all recipes below
   - [`cargo-nextest`](https://nexte.st/) — faster test runner
   - [`git-cliff`](https://git-cliff.org) — changelog generator (`cargo install git-cliff --locked`)
4. Make your changes on a feature branch.

On Linux, building the GPUI adapter requires the usual GUI development
libraries:

```bash
sudo apt-get install -y libxkbcommon-dev libx11-dev libxcb1-dev \
  libwayland-dev libfontconfig1-dev libssl-dev
```

## Development Workflow

Use the `just` recipes — they mirror what CI runs:

- Run all tests: `just test` (uses cargo-nextest)
- Check formatting: `just fmt-check` (auto-format with `just fmt`)
- Run lints without GPUI system deps: `just check`
- Run lints including the GPUI crates: `just check-all`
- Build docs: `cargo doc --no-deps --workspace`

CI (`.github/workflows/ci.yml`) runs format check, clippy with `-D warnings`,
the unit + conformance tests per adapter, the trybuild macro UI tests
(Leptos, Dioxus, GPUI), and a compile check of every example. A PR must keep
all of these green.

### Clippy runs one compilation unit at a time

Never run a workspace-wide `cargo clippy --workspace --all-features`: the
framework features on `quoin-macros` / `quoin-macros-core` are **mutually
exclusive by contract** (enabling two backends is a deliberate, documented
compile error — see `quoin-macros/src/lib.rs`). A single workspace-wide
invocation builds `quoin-floem` and `quoin-xilem` together, cargo unifies
their feature sets on the macro crates, and the mutual-exclusion diagnostic
fires before any lint does. Lint each unit separately — `just lint` and
`just lint-pedantic` encode the exact split CI uses (framework-free batch,
one invocation per backend feature, one per adapter crate).

### Debugging macro expansions

Set `QUOIN_DEBUG_EXPAND=1` while building any crate that invokes the quoin
macros: every `quoin_render!` then fails its compilation **on purpose** and
prints its full expansion as the error message
(`QUOIN_DEBUG_EXPANSION: …`). This is a debugging aid, not a supported
build mode — unset the variable to build normally:

```bash
QUOIN_DEBUG_EXPAND=1 cargo build -p your-crate 2>&1 | grep QUOIN_DEBUG_EXPANSION
```

### Zero-overhead gates

The Floem/Xilem backends are guarded by three gates (run `just gate`):

- **Expansion gate** — static trees expand to native calls only (no boxing,
  `Vec`, `Rc`/`Arc`, `dyn`); dynamic seams box exactly once.
- **Allocation gate** — the expansion of a static tree allocates exactly
  what the hand-written equivalent allocates (dhat-counted; the Floem gate
  takes the minimum over repeated constructions because floem's reactive
  runtime defers work into later measurement windows).
- **Binary gate** (opt-in, needs `cargo-bloat`) — `just gate-bloat` diffs
  `counter-xilem`'s binary against `docs/bloat-baseline.txt`; capture a new
  baseline deliberately with `just gate-bloat-save`.

The parity contract between the emitters and `docs/BACKEND-PARITY.md` is
enforced by `quoin-macros-core/tests/parity_matrix.rs`.

## Commit Messages (Conventional Commits)

This project uses [Conventional Commits](https://www.conventionalcommits.org/):
`CHANGELOG.md` is generated automatically by [git-cliff](https://git-cliff.org)
from the commit history, so non-conventional messages will not appear in the
release notes.

Use the following prefixes:

| Prefix | Changelog section | Example |
|--------|-------------------|---------|
| `feat:` | 🚀 Added | `feat(ui): add separator element` |
| `fix:` | 🔧 Fixed | `fix(gpui): abort spawned task on handle drop` |
| `perf:` | ⚡ Performance | `perf(core): bound cancellation waker registry` |
| `doc:` | 📚 Documentation | `doc: document executor lifecycle semantics` |
| `test:` | 🧪 Tests | `test: add abort/detach conformance tests` |
| `refactor:` | ♻️ Changed | `refactor(macros): share handler wrapping` |
| `chore:` | 🧹 Cleanup | `chore: update dependencies` |

Mark breaking changes with `!` (e.g. `feat!: rename Executor::spawn`) —
git-cliff always keeps them, even when the section would otherwise be skipped.

Regenerate the changelog locally with `just changelog`; releases are cut by
pushing a `vX.Y.Z` tag, which triggers `.github/workflows/release.yml`.

## Adding a New Adapter

1. Create a new crate `quoin-<framework>` in the workspace.
2. Implement `quoin::ReactiveContext` for your framework's reactive context,
   and implement `Executor`/`JoinHandle` following the lifecycle semantics
   documented in `quoin-core` (`abort`, `detach`, cancel-on-drop).
3. Add a conformance test using `quoin-conformance`
   (`define_conformance_tests!(sync, YourHarness)`, or the `gpui` form for
   async frameworks).
4. Add a README following the pattern of existing adapters.
5. Open a PR!

## Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By participating in this project you agree to abide by its terms.

## License

By contributing, you agree that your contributions will be licensed under the MIT license that covers the project.
