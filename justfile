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

# Run UI macro expansion tests (trybuild) for quoin-macros-tests
test-quoin-macros-gpui:
    cargo nextest run -p quoin-macros-tests --features gpui

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

lint:
    cargo clippy --all-targets -p quoin-macros-core -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features gpui -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features leptos -- -D warnings
    cargo clippy --all-targets -p quoin-macros-tests --features dioxus -- -D warnings
    cargo clippy --all-targets -- -D warnings

check: fmt-check lint

fix:
    cargo fix --allow-dirty --all-targets

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

# ---------------------------------------------------------------------
# Compile-check all examples (no windows opened)
# ---------------------------------------------------------------------

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

wr:
    watchexec -w ./wr.sh --clear -r "sh ./wr.sh"
