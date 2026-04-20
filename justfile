# justfile for quoin – framework-agnostic reactive core examples

set shell := ["bash", "-c"]

# Build the entire workspace
build:
    cargo build

# Run all conformance tests
test:
    cargo nextest run --all

# Clean the workspace
clean:
    cargo clean

# ------------------------------------------------------------
# Counter Examples
# ------------------------------------------------------------

# Run GPUI counter (native)
run-gpui:
    cargo run -p counter-gpui

# Run Dioxus counter (native) - temporarily disables GPUI crates to avoid cocoa conflict
run-dioxus:
    cd examples/counter-dioxus && cargo run

# Run Leptos counter (native SSR server)
run-leptos:
    cargo leptos serve -p counter-leptos

# Serve Leptos counter (WASM client) with Trunk
serve-leptos:
    cd examples/counter-leptos && trunk serve

# ------------------------------------------------------------
# Development Helpers
# ------------------------------------------------------------

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

# ------------------------------------------------------------
# Leptos SSR (Native) Helpers
# ------------------------------------------------------------

# Clean Leptos build artifacts
leptos-clean:
    cargo leptos clean -p counter-leptos

# Build Leptos SSR server (without running)
leptos-build:
    cargo leptos build -p counter-leptos

# ------------------------------------------------------------
# Full Demo (all examples in sequence – for verification)
# ------------------------------------------------------------

demo:
    @echo "=== GPUI Counter ==="
    @cargo run -p counter-gpui &
    @sleep 2
    @echo "=== Dioxus Counter ==="
    @cargo run -p counter-dioxus &
    @echo "=== Leptos (SSR) starting on http://127.0.0.1:3000 ==="
    @cargo leptos serve -p counter-leptos
