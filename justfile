# justfile for quoin – framework-agnostic reactive core with macros and UCP

set shell := ["bash", "-c"]

# ---------------------------------------------------------------------
# Build & Clean
# ---------------------------------------------------------------------

# Build the entire workspace
build:
    cargo build

# Build with all features (where applicable)
build-all:
    cargo build --all-features

# Clean the workspace
clean:
    cargo clean

# ---------------------------------------------------------------------
# Testing with cargo-nextest (recommended)
# ---------------------------------------------------------------------

# Run all tests across the workspace
test:
    cargo nextest run --all

# Run tests for the core quoin crate
test-quoin:
    cargo nextest run -p quoin

# Run tests for quoin-macros (requires feature flag)
test-quoin-macros-gpui:
    cargo nextest run -p quoin-macros --features gpui

test-quoin-macros-leptos:
    cargo nextest run -p quoin-macros --features leptos

test-quoin-macros-dioxus:
    cargo nextest run -p quoin-macros --features dioxus

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

# Run conformance tests (all adapters)
test-conformance:
    cargo nextest run -p quoin-conformance

# Run UI macro expansion tests (trybuild) for quoin-macros
test-macros-ui-gpui:
    cargo test -p quoin-macros --features gpui --test macro_tests

test-macros-ui-leptos:
    cargo test -p quoin-macros --features leptos --test macro_tests

test-macros-ui-dioxus:
    cargo test -p quoin-macros --features dioxus --test macro_tests

# Run all macro UI tests (all features)
test-macros-ui-all: test-macros-ui-gpui test-macros-ui-leptos test-macros-ui-dioxus

# ---------------------------------------------------------------------
# Counter Examples
# ---------------------------------------------------------------------

# Run GPUI counter (native)
run-gpui:
    cargo run -p counter-gpui

# Run Dioxus counter (native) - temporarily disables GPUI crates to avoid cocoa conflict
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
    cargo run -p counter-floem

# Run Xilem counter (native)
run-xilem:
    cargo run -p counter-xilem

# ---------------------------------------------------------------------
# Development Helpers
# ---------------------------------------------------------------------

# Check formatting and lints
check:
    cargo fmt --check
    cargo clippy --all-targets --all-features -- -D warnings

# Fix formatting
fmt:
    cargo fmt

# Run cargo fix for all packages
fix:
    cargo fix --allow-dirty --all-targets

# Watch for changes and run GPUI example (requires cargo-watch)
watch-gpui:
    cargo watch -x 'run -p counter-gpui'

# Watch and run Dioxus example
watch-dioxus:
    cargo watch -x 'run -p counter-dioxus'

# Watch and run Floem example
watch-floem:
    cargo watch -x 'run -p counter-floem'

# Watch and run Xilem example
watch-xilem:
    cargo watch -x 'run -p counter-xilem'

# ---------------------------------------------------------------------
# Leptos SSR (Native) Helpers
# ---------------------------------------------------------------------

# Clean Leptos build artifacts
leptos-clean:
    cargo leptos clean -p counter-leptos

# Build Leptos SSR server (without running)
leptos-build:
    cargo leptos build -p counter-leptos

# ---------------------------------------------------------------------
# Full Demo (all examples in sequence – for verification)
# ---------------------------------------------------------------------

demo:
    @echo "=== GPUI Counter ==="
    @cargo run -p counter-gpui &
    @sleep 2
    @echo "=== Dioxus Counter ==="
    @cargo run -p counter-dioxus &
    @sleep 2
    @echo "=== Floem Counter ==="
    @cargo run -p counter-floem &
    @sleep 2
    @echo "=== Xilem Counter ==="
    @cargo run -p counter-xilem &
    @sleep 2
    @echo "=== Leptos (SSR) starting on http://127.0.0.1:3000 ==="
    @cargo leptos serve -p counter-leptos

# ---------------------------------------------------------------------
# Utility: Run all examples (background, no waiting)
# ---------------------------------------------------------------------

run-all:
    cargo run -p counter-gpui &
    cd examples/counter-dioxus && cargo run &
    cargo run -p counter-floem &
    cargo run -p counter-xilem &
    cargo leptos serve -p counter-leptos &
