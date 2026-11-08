# justfile for quoin – framework-agnostic reactive core with macros and UCP

set shell := ["bash", "-c"]

# ---------------------------------------------------------------------
# Build & Clean
# ---------------------------------------------------------------------

build:
    cargo build

clean:
    cargo clean

# ---------------------------------------------------------------------
# Testing with cargo-nextest (recommended)
# ---------------------------------------------------------------------

test:
    cargo nextest run --all

test-quoin:
    cargo nextest run -p quoin

test-quoin-macros:
    cargo nextest run -p quoin-macros-core --no-tests pass

# Testing & devtools crates
test-quoin-testkit:
    cargo nextest run -p quoin-testkit

test-quoin-devtools:
    cargo nextest run -p quoin-devtools

# Headless e2e examples (todo store, async resource flows, devtools session)
test-headless-examples:
    cargo nextest run -p todo-core -p fetch-core -p devtools-demo

# Micro-benchmarks for the core primitives (release profile)
bench-core:
    cargo run --release -p quoin-core --example bench

# Micro-benchmarks for the Xilem effect path — the runtime the effect!
# macro produces on that backend (release profile)
bench-xilem:
    cargo run --release -p quoin-xilem --example bench-effects

# Micro-benchmarks for the devtools event log (release profile)
bench-devtools:
    cargo run --release -p quoin-devtools --example bench-devtools

# Micro-benchmarks for the web-backend component! expansion path — the
# keys-diagnostic branch is pinned as unmeasurable (release profile)
bench-web-expansion:
    cargo run --release -p quoin-macros-core --features leptos,dioxus --example bench-web-expansion

# Micro-benchmarks for the Leptos effect path — the core effect graph
# observed through the bridged LeptosSignal (release profile)
bench-leptos:
    cargo run --release -p quoin-leptos --example bench-effects

# Micro-benchmarks for the Dioxus effect path — the core effect graph
# observed through the bridged DioxusSignal (release profile)
bench-dioxus:
    cargo run --release -p quoin-dioxus --example bench-effects

# Micro-benchmarks for the Floem effect path — the core effect graph
# observed through the bridged FloemSignal (release profile)
bench-floem:
    cargo run --release -p quoin-floem --example bench-effects

# Headless devtools demo with tracing enabled
trace-demo:
    QUOIN_TRACE=1 cargo run -p devtools-demo

# Visual counter demo with the live devtools trace panel (Xilem window)
trace-counter-xilem:
    cargo run --manifest-path examples/counter-xilem/Cargo.toml

# Run UI macro expansion tests (trybuild) for quoin-macros-tests
test-quoin-macros-gpui:
    cargo nextest run -p quoin-macros-tests --features gpui

# gpui-rsx-classes engine (docs/gpui-rsx-classes.md): unit tests + parity
# gates + expansion compile-check against the vendored GPUI stub. Fast — no
# real gpui-pre build.
test-gpui-rsx-classes:
    cargo test -p quoin-macros-core --features gpui,gpui-rsx-classes
    cargo test -p quoin-interop-tests
    cargo test -p quoin-grsx-compiletest
    cargo run -p quoin-grsx-compiletest

# The flag layered on the real GPUI stack (trybuild + conformance) for CI.
test-quoin-macros-gpui-rsx-classes:
    cargo nextest run -p quoin-macros-tests --features gpui,gpui-rsx-classes

test-quoin-macros-leptos:
    cargo nextest run -p quoin-macros-tests --features leptos

test-quoin-macros-dioxus:
    cargo nextest run -p quoin-macros-tests --features dioxus

# Run tests for framework adapters
test-quoin-gpui:
    cargo nextest run -p quoin-gpui

test-quoin-leptos:
    cargo nextest run -p quoin-leptos

test-quoin-dioxus:
    cargo nextest run -p quoin-dioxus

test-quoin-floem:
    cargo nextest run -p quoin-floem

test-quoin-xilem:
    cargo nextest run -p quoin-xilem

# Run conformance tests per adapter
# Note: GPUI conformance runs inside quoin-conformance (has gpui dep).

# Leptos/Dioxus/Floem/Xilem conformance runs inside their adapter crates.
test-conformance-gpui:
    cargo nextest run -p quoin-conformance --features gpui

test-conformance-leptos:
    cargo nextest run -p quoin-leptos

test-conformance-dioxus:
    cargo nextest run -p quoin-dioxus

test-conformance-floem:
    cargo nextest run -p quoin-floem

test-conformance-xilem:
    cargo nextest run -p quoin-xilem

# Run all macro UI tests (sequentially, one feature at a time)
test-macros-ui-all: test-quoin-macros-gpui test-quoin-macros-leptos test-quoin-macros-dioxus

# ---------------------------------------------------------------------
# Counter Examples (excluded from workspace — use --manifest-path)
# ---------------------------------------------------------------------

run-counter-gpui:
    cargo run --manifest-path examples/counter-gpui/Cargo.toml

run-counter-dioxus:
    cd examples/counter-dioxus && cargo run

run-counter-leptos:
    cd examples/counter-leptos && trunk serve

run-counter-floem:
    cargo run --manifest-path examples/counter-floem/Cargo.toml

run-counter-xilem:
    cargo run --manifest-path examples/counter-xilem/Cargo.toml

# ---------------------------------------------------------------------
# UCP Examples (excluded from workspace — use --manifest-path)
# ---------------------------------------------------------------------

run-ucp-gpui:
    cargo run --manifest-path examples/ucp-demo-gpui/Cargo.toml

run-ucp-dioxus:
    cd examples/ucp-demo-dioxus && cargo run

run-ucp-leptos:
    cd examples/ucp-demo-leptos && trunk serve

run-ucp-leptos-shadcn:
    cd examples/ucp-demo-leptos-shadcn && trunk serve

build-ucp-lib framework="gpui":
    cargo build --manifest-path examples/ucp-lib/Cargo.toml --features {{ framework }}

# ---------------------------------------------------------------------
# Mini Devtools Example
# ---------------------------------------------------------------------

run-mini-devtools:
    cargo run --manifest-path examples/mini-devtools-gpui/Cargo.toml

watch-mini-devtools:
    cargo watch --manifest-path examples/mini-devtools-gpui/Cargo.toml -x run

# ---------------------------------------------------------------------
# Development Helpers
# ---------------------------------------------------------------------

fmt-check:
    cargo fmt --check

fmt:
    cargo fmt

# Clippy for the framework-free crates (no GPUI system deps needed).
lint-core:
    cargo clippy --all-targets -p quoin-core -p quoin-macros-core -p quoin-macros -p quoin-ui -p quoin-conformance -p quoin-testkit -p quoin-devtools -- -D warnings

# Clippy for the GPUI adapter crates (requires GPUI system libraries).
lint-gpui:
    cargo clippy --all-targets -p quoin-gpui -p quoin-ui-gpui -- -D warnings

# Maximum-strictness lint: pedantic + nursery lints on top of the workspace
# policy, per compilation unit and without re-linting path dependencies
# (`--no-deps`) — for the same mutual-exclusion reason `lint` is split. The
# units below are held clean at this level with zero `allow` attributes
# (enforced by the workspace `allow_attributes = deny`); quoin-macros-core
# and quoin-ui still carry pedantic-level findings (tracked for a follow-up
# pass) and are held at the default level by `just lint`.
lint-pedantic:
    cargo clippy --all-targets --no-deps -p quoin-core -p quoin-macros -p quoin-conformance -p quoin-testkit -p quoin-devtools -p quoin-style -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo clippy --all-targets --no-deps -p quoin-macros-tests --features leptos -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo clippy --all-targets --no-deps -p quoin-macros-tests --features dioxus -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo clippy --all-targets --no-deps -p quoin-floem -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo clippy --all-targets --no-deps -p quoin-xilem -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo clippy --all-targets --no-deps -p quoin-leptos -- -D warnings -W clippy::pedantic -W clippy::nursery
    cargo clippy --all-targets --no-deps -p quoin-dioxus -- -D warnings -W clippy::pedantic -W clippy::nursery

# Full lint matrix: one cargo invocation per compilation unit. The framework
# features on quoin-macros(-core) are mutually exclusive by contract (see
# quoin-macros/src/lib.rs), so a single workspace-wide clippy run would unify
# `floem` + `xilem` feature sets and trip the deliberate mutual-exclusion
# diagnostic before any lint fires; CI encodes the same per-unit split.
lint:
    cargo clippy --all-targets -p quoin-macros-core -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features gpui -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features leptos -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features dioxus -- -D warnings
    cargo clippy --all-targets -p quoin-core -p quoin-macros -p quoin-ui -p quoin-conformance -p quoin-testkit -p quoin-devtools -p quoin-style -- -D warnings
    cargo clippy --all-targets -p quoin-floem -- -D warnings
    cargo clippy --all-targets -p quoin-xilem -- -D warnings
    cargo clippy --all-targets -p quoin-leptos -- -D warnings
    cargo clippy --all-targets -p quoin-dioxus -- -D warnings

# Fast pre-commit check: format + framework-free clippy.
check: fmt-check lint-core

# Full check including GPUI crates (needs libxkbcommon-dev etc.).
check-all: fmt-check lint

fix:
    cargo fix --allow-dirty --all-targets

# ---------------------------------------------------------------------
# Changelog (git-cliff)
# ---------------------------------------------------------------------

# Regenerate CHANGELOG.md from the Conventional Commit history.
# Requires git-cliff: cargo install git-cliff --locked
changelog:
    git cliff --verbose --output CHANGELOG.md

# Print only the changelog for the most recent version (used for release notes).
release-notes:
    git cliff --latest --strip all

watch-counter-gpui:
    cargo watch --manifest-path examples/counter-gpui/Cargo.toml -x run

watch-counter-dioxus:
    cd examples/counter-dioxus && cargo watch -x run

watch-counter-leptos:
    cd examples/counter-leptos && cargo watch -x "trunk serve"

watch-counter-floem:
    cargo watch --manifest-path examples/counter-floem/Cargo.toml -x run

watch-counter-xilem:
    cargo watch --manifest-path examples/counter-xilem/Cargo.toml -x run

watch-ucp-gpui:
    cargo watch --manifest-path examples/ucp-demo-gpui/Cargo.toml -x run

watch-ucp-dioxus:
    cd examples/ucp-demo-dioxus && cargo watch -x run

watch-ucp-leptos:
    cd examples/ucp-demo-leptos && cargo watch -x "trunk serve"

watch-ucp-leptos-shadcn:
    cd examples/ucp-demo-leptos-shadcn && cargo watch -x "trunk serve"

# ---------------------------------------------------------------------
# Leptos SSR (Native) Helpers
# ---------------------------------------------------------------------

leptos-clean:
    cargo leptos clean -p counter-leptos

leptos-build:
    cargo leptos build -p counter-leptos

build-examples:
    cargo build --manifest-path examples/counter-gpui/Cargo.toml
    cargo build --manifest-path examples/counter-dioxus/Cargo.toml
    cargo build --manifest-path examples/counter-leptos/Cargo.toml
    cargo build --manifest-path examples/counter-floem/Cargo.toml
    cargo build --manifest-path examples/counter-xilem/Cargo.toml
    cargo build --manifest-path examples/ucp-demo-gpui/Cargo.toml
    cargo build --manifest-path examples/ucp-demo-dioxus/Cargo.toml
    cargo build --manifest-path examples/ucp-demo-leptos/Cargo.toml
    cargo build --manifest-path examples/ucp-demo-leptos-shadcn/Cargo.toml
    cargo build --manifest-path examples/mini-devtools-gpui/Cargo.toml
    cargo build --manifest-path examples/ucp-lib/Cargo.toml --features gpui
    cargo build --manifest-path examples/ucp-lib/Cargo.toml --features leptos
    cargo build --manifest-path examples/ucp-lib/Cargo.toml --features dioxus
    cargo build -p todo-core -p fetch-core -p devtools-demo --all-targets
    @echo "Note: ucp-demo-leptos-shadcn requires shadcn deps — check crate availability"
    @echo "All examples build OK"

# ---------------------------------------------------------------------
# Full Demo (all examples in sequence – for verification)
# ---------------------------------------------------------------------

demo:
    @echo "=== GPUI Counter ==="
    @cargo run --manifest-path examples/counter-gpui/Cargo.toml &
    @sleep 2
    @echo "=== Dioxus Counter ==="
    @cd examples/counter-dioxus && cargo run &
    @sleep 2
    @echo "=== Floem Counter ==="
    @cargo run --manifest-path examples/counter-floem/Cargo.toml &
    @sleep 2
    @echo "=== Xilem Counter ==="
    @cargo run --manifest-path examples/counter-xilem/Cargo.toml &
    @sleep 2
    @echo "=== Leptos (SSR) starting on http://127.0.0.1:3000 ==="
    @cargo leptos serve -p counter-leptos

# ---------------------------------------------------------------------
# Utility: Run all examples (background, no waiting)
# ---------------------------------------------------------------------

run-all:
    cargo run --manifest-path examples/counter-gpui/Cargo.toml &
    cd examples/counter-dioxus && cargo run &
    cargo run --manifest-path examples/counter-floem/Cargo.toml &
    cargo run --manifest-path examples/counter-xilem/Cargo.toml &
    cd examples/counter-leptos && trunk serve &
    cargo run --manifest-path examples/ucp-demo-gpui/Cargo.toml &
    cargo run --manifest-path examples/mini-devtools-gpui/Cargo.toml &

# ---------------------------------------------------------------------
# Development Helpers (Phase 5)
# ---------------------------------------------------------------------

# Expand macros for inspection
expand component="ucp-lib" feature="gpui":
    cargo expand --manifest-path examples/{{ component }}/Cargo.toml \
        --features {{ feature }}

# Run global-state conformance tests
test-global:
    cargo nextest run -p quoin-conformance --no-tests pass --features leptos -- provide_and_use_global
    cargo nextest run -p quoin-conformance --no-tests pass --features gpui   -- provide_and_use_global

# Clear trybuild cache (use when tests show stale errors)
clean-trybuild:
    rm -rf target/tests/trybuild

# ---------------------------------------------------------------------
# Zero-overhead gates (rendering-parity plan, section 3)
# ---------------------------------------------------------------------

# Gate 1 — expansion gate: no boxing/Vec/dyn in fully static trees, exactly
# one seam per dynamic construct. Runs on both dynamic backends.
gate-expand:
    cargo nextest run -p quoin-macros-core --features floem --test floem_expansion
    cargo nextest run -p quoin-macros-core --features xilem --test xilem_expansion
    cargo nextest run -p quoin-macros-core --features floem,xilem --test parity_matrix

# Gate 2 — allocation gate: the quoin_render! expansion of a fully static
# tree must allocate exactly what the hand-written equivalent allocates.
# (Floem measures the minimum over repeated constructions — its reactive
# runtime defers work, so single-shot deltas are noisy; see the gate's docs.)
gate-alloc:
    cargo nextest run -p quoin-xilem --test allocation_gate
    cargo nextest run -p quoin-floem --test allocation_gate

# Gate 3 — binary gate (requires cargo-bloat): the macro must add zero bytes
# over the hand-written view. Compares against docs/bloat-baseline.txt;
# regenerate the baseline intentionally with `just gate-bloat-save`. The first
# run on a fresh checkout saves the baseline instead of failing on a missing
# file.
gate-bloat:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v cargo-bloat >/dev/null 2>&1; then
        echo "gate-bloat requires cargo-bloat (cargo install cargo-bloat); skipping"
        exit 0
    fi
    cargo bloat --release --manifest-path examples/counter-xilem/Cargo.toml --crates > /tmp/bloat-now.txt
    if [ ! -f docs/bloat-baseline.txt ]; then
        echo "no docs/bloat-baseline.txt yet; saving the current build as the baseline"
        cp /tmp/bloat-now.txt docs/bloat-baseline.txt
        exit 0
    fi
    diff docs/bloat-baseline.txt /tmp/bloat-now.txt \
        || (echo "binary size changed; re-review and run \`just gate-bloat-save\`" && exit 1)

gate-bloat-save:
    cargo bloat --release --manifest-path examples/counter-xilem/Cargo.toml --crates > docs/bloat-baseline.txt

# All gates that run without extra tooling.
gate: gate-expand gate-alloc
