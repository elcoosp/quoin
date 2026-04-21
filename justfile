# justfile for quoin – framework-agnostic reactive core with macros and UCP

set shell := ["bash", "-c"]

# ---------------------------------------------------------------------
# Build & Clean
# ---------------------------------------------------------------------

# Build the workspace (library crates only)
build:
    cargo build

# Clean the workspace
clean:
    cargo clean

# ---------------------------------------------------------------------
# Testing with cargo-nextest (recommended)
# ---------------------------------------------------------------------

# Run all workspace tests (library crates — examples excluded)
test:
    cargo nextest run --all

# Run tests for the core quoin crate
test-quoin:
    cargo nextest run -p quoin

# Run tests for quoin-macros-core (no features needed)
test-quoin-macros:
    cargo nextest run -p quoin-macros-core

# Run UI macro expansion tests (trybuild) for quoin-macros-tests
test-quoin-macros-gpui:
    cargo nextest run -p quoin-macros-tests

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

# Run conformance tests per adapter (each feature separately)
test-conformance-gpui:
    cargo nextest run -p quoin-conformance --features gpui

test-conformance-leptos:
    cargo nextest run -p quoin-conformance --features leptos

test-conformance-dioxus:
    cargo nextest run -p quoin-conformance --features dioxus

# Run all macro UI tests (sequentially, one feature at a time)
test-macros-ui-all: test-quoin-macros-gpui test-quoin-macros-leptos test-quoin-macros-dioxus

# ---------------------------------------------------------------------
# Counter Examples (excluded from workspace — use --manifest-path)
# ---------------------------------------------------------------------

# Run GPUI counter (native)
run-gpui:
    cargo run --manifest-path examples/counter-gpui/Cargo.toml

# Run Dioxus counter (native) — standalone to avoid cocoa conflict
run-dioxus:
    cd examples/counter-dioxus && cargo run

# Run Leptos counter (SSR server)
run-leptos:
    cargo leptos serve -p counter-leptos

# Serve Leptos counter (WASM client) with Trunk
serve-leptos:
    cd examples/counter-leptos && trunk serve

# Run Floem counter (native)
run-floem:
    cargo run --manifest-path examples/counter-floem/Cargo.toml

# Run Xilem counter (native)
run-xilem:
    cargo run --manifest-path examples/counter-xilem/Cargo.toml

# ---------------------------------------------------------------------
# UCP Examples (excluded from workspace — use --manifest-path)
# ---------------------------------------------------------------------

# Run GPUI UCP demo (native)
run-ucp-gpui:
    cargo run --manifest-path examples/ucp-demo-gpui/Cargo.toml

# Run Dioxus UCP demo (native) — standalone to avoid cocoa conflict
run-ucp-dioxus:
    cd examples/ucp-demo-dioxus && cargo run

# Build UCP lib for a specific framework

# Usage: just build-ucp-lib gpui
build-ucp-lib framework="gpui":
    cargo build --manifest-path examples/ucp-lib/Cargo.toml --features {{ framework }}

# ---------------------------------------------------------------------
# Mini Devtools Example
# ---------------------------------------------------------------------

# Run GPUI Mini Devtools (native)
run-mini-devtools:
    cargo run --manifest-path examples/mini-devtools-gpui/Cargo.toml

# Watch and run GPUI Mini Devtools
watch-mini-devtools:
    cargo watch --manifest-path examples/mini-devtools-gpui/Cargo.toml -x run

# ---------------------------------------------------------------------
# Development Helpers
# ---------------------------------------------------------------------

# Check formatting
fmt-check:
    cargo fmt --check

# Fix formatting
fmt:
    cargo fmt

# Lint workspace (per-feature to avoid feature unification errors)
lint:
    cargo clippy --all-targets -p quoin-macros-core -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features gpui -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features leptos -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features dioxus -- -D warnings
    cargo clippy --all-targets -- -D warnings

# Format + lint (full check)
check: fmt-check lint

# Run cargo fix for all packages
fix:
    cargo fix --allow-dirty --all-targets

# Watch for changes and run (requires cargo-watch)
watch-gpui:
    cargo watch --manifest-path examples/counter-gpui/Cargo.toml -x run

watch-dioxus:
    cd examples/counter-dioxus && cargo watch -x run

watch-floem:
    cargo watch --manifest-path examples/counter-floem/Cargo.toml -x run

watch-xilem:
    cargo watch --manifest-path examples/counter-xilem/Cargo.toml -x run

watch-ucp-gpui:
    cargo watch --manifest-path examples/ucp-demo-gpui/Cargo.toml -x run

watch-ucp-dioxus:
    cd examples/ucp-demo-dioxus && cargo watch -x run

# ---------------------------------------------------------------------
# Leptos SSR (Native) Helpers
# ---------------------------------------------------------------------

leptos-clean:
    cargo leptos clean -p counter-leptos

leptos-build:
    cargo leptos build -p counter-leptos

# ---------------------------------------------------------------------
# Compile-check all examples (no windows opened)
# ---------------------------------------------------------------------

build-examples:
    cargo build --manifest-path examples/counter-gpui/Cargo.toml
    cargo build --manifest-path examples/ucp-demo-gpui/Cargo.toml
    cargo build --manifest-path examples/mini-devtools-gpui/Cargo.toml
    cargo build --manifest-path examples/ucp-lib/Cargo.toml --features gpui
    cargo build --manifest-path examples/counter-leptos/Cargo.toml
    cargo build --manifest-path examples/ucp-demo-leptos/Cargo.toml
    cargo build --manifest-path examples/counter-floem/Cargo.toml
    cargo build --manifest-path examples/counter-xilem/Cargo.toml
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
    cargo leptos serve -p counter-leptos &
    cargo run --manifest-path examples/ucp-demo-gpui/Cargo.toml &
    cargo run --manifest-path examples/mini-devtools-gpui/Cargo.toml &
