# Changelog

All notable changes to Quoin will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-04-22

### 🚀 Added – New features and capabilities

#### Core
- **`ReactiveContext` trait**: A unified entry point for creating reactive signals, spawning async tasks, and managing global state across all supported frameworks. This is the foundation for writing framework-agnostic business logic. ([13ba55c](https://github.com/elcoosp/quoin/commit/13ba55c747915030959f70b9bee2fecf81f5e0dd), [d22dee4](https://github.com/elcoosp/quoin/commit/d22dee4e3fdab5aed6cedbbc690548c1901a98fb))
- **`Signal<T>` trait**: Readable and writable reactive values with `get()`, `set()`, `update()`, and `with()` for borrowing without cloning. Signals are `Clone` and can be freely passed around. ([55f37f7](https://github.com/elcoosp/quoin/commit/55f37f7f32db0c2fe8c19cd7753786fa13802df8), [2bdf6d3](https://github.com/elcoosp/quoin/commit/2bdf6d39719ca1c220a147895255848aba584ba2))
- **`Executor` and `JoinHandle`**: Async task spawning abstraction that works across all supported UI frameworks. ([d22dee4](https://github.com/elcoosp/quoin/commit/d22dee4e3fdab5aed6cedbbc690548c1901a98fb))
- **`CancellationToken`**: Cooperative cancellation for long-running async tasks, preventing resource leaks when components unmount. ([13ba55c](https://github.com/elcoosp/quoin/commit/13ba55c747915030959f70b9bee2fecf81f5e0dd))
- **Convenience macros**: `read!` for quick signal access and `action!` for creating move closures with automatic cloning. ([3fcba94](https://github.com/elcoosp/quoin/commit/3fcba945faca7a02a96474ddf25290cd16d5d321))

#### Framework Adapters
- **GPUI** (`quoin-gpui`): Full `ReactiveContext` implementation with automatic UI updates via view notifiers. Signal mutations trigger repaints automatically when connected with `set_view_update_notifier`. ([551147c](https://github.com/elcoosp/quoin/commit/551147cc31be02a113f87314f9aa0dd298ccbc58), [0bb0fb9](https://github.com/elcoosp/quoin/commit/0bb0fb9f4b786956177a48f87190ab0f62e634fc))
- **Leptos** (`quoin-leptos`): CSR/SSR support with `SendWrapper` for thread safety. Works seamlessly with Leptos 0.8. ([1800728](https://github.com/elcoosp/quoin/commit/180072848aacccb35faf7b23f0590ab52f1e9b1b), [8c9c25e](https://github.com/elcoosp/quoin/commit/8c9c25ecb5ecb03249981ae4879123a9899b52ad))
- **Dioxus** (`quoin-dioxus`): Signals backed by Dioxus' reactive system. Desktop and web targets supported. ([4d9368e](https://github.com/elcoosp/quoin/commit/4d9368e5912f052cbb51be5fb452a045d5e0feaf), [0d12bac](https://github.com/elcoosp/quoin/commit/0d12bac7884da3f174665af9633dccf471fb91bd))
- **Xilem** (`quoin-xilem`): Tokio runtime integration with `XilemContext` for reactive updates. ([0c969a1](https://github.com/elcoosp/quoin/commit/0c969a1fb3280344a8878ec13a990252a37ecd05), [0fc2180](https://github.com/elcoosp/quoin/commit/0fc218073da17f60de1fd4df8e8437521ded4675))
- **Floem** (`quoin-floem`): Reactive signals using Floem's native `RwSignal`. ([39a8659](https://github.com/elcoosp/quoin/commit/39a865931a0b9ec93bdcfad3bb414a3cfb5bc311))

#### Declarative Macros
- **`component!`**: Define a component once and render it with any framework. Supports props, state, globals, actions, and lifecycle hooks (`on_mount`, `on_unmount`). ([8e9e68a](https://github.com/elcoosp/quoin/commit/8e9e68aec699c98af53e53306b2a26ede60c646e))
- **`quoin_render!`**: Write UI with a JSX-like syntax and Tailwind classes. Tailwind classes are automatically transpiled to GPUI method chains, while Leptos and Dioxus use native `view!`/`rsx!` macros. ([dcc0ebf](https://github.com/elcoosp/quoin/commit/dcc0ebf713be8648e2be00839d5b7826f39d25bb))
- **`effect!`**: Run reactive side effects when signals change, with optional cleanup. Available in both legacy `watch:` syntax and new structured `deps:`, `run:`, `cleanup:` syntax. ([18386b7](https://github.com/elcoosp/quoin/commit/18386b7321b30e3ee115d9a87d65c207b29658e2))
- **`run_app!`**: One-line application bootstrap that automatically sets up the correct window, context, and reactive wiring for each framework. ([27952a9](https://github.com/elcoosp/quoin/commit/27952a9a5ea1d3cb9c4dbaa17e514eb2a4f7e062))
- **`quoin_element!`**: Register custom elements that can be used inside `quoin_render!`. ([09e1f3c](https://github.com/elcoosp/quoin/commit/09e1f3cfb73029156e6ef3d70ef05ccda80c94ea))

#### Universal Component Protocol (UCP)
- **`quoin-ui`**: Abstract traits for complex UI components: `VirtualListAdapter`, `TableAdapter`, `TextInputAdapter`, `ButtonAdapter`, `DropdownMenuAdapter`, `TabBarAdapter`, `Clipboard`, `QuoinTheme`, and `Navigator`. ([27952a9](https://github.com/elcoosp/quoin/commit/27952a9a5ea1d3cb9c4dbaa17e514eb2a4f7e062))
- **`quoin-ui-gpui`**: GPUI backend for UCP with `QuoinInputManager` for automatic two-way input binding. ([27952a9](https://github.com/elcoosp/quoin/commit/27952a9a5ea1d3cb9c4dbaa17e514eb2a4f7e062), [c51efd5](https://github.com/elcoosp/quoin/commit/c51efd5890688eebbae7a7d6c8d4e04496696049))

#### Special Components in `quoin_render!`
- **`tabs` / `tab`**: Tab navigation with active state tracking. ([9699fa1](https://github.com/elcoosp/quoin/commit/9699fa1619959653f29181b7a00982c3052e634a))
- **`data_table` / `column`**: Sortable, striped data tables with custom cell rendering. ([bf4e93e](https://github.com/elcoosp/quoin/commit/bf4e93ec5d1ee9241ef8e38aee7e856d26d8cc4e), [92fbfdd](https://github.com/elcoosp/quoin/commit/92fbfdd95db8e8e7e3fc959c2853c00a718c35f7))
- **`virtual_list`**: Efficient rendering of large scrollable lists. ([de086df](https://github.com/elcoosp/quoin/commit/de086dfe91fb2007b6c51b06056c579d61234aba))
- **`dropdown_menu` / `item`**: Popup menus with click handlers. ([de086df](https://github.com/elcoosp/quoin/commit/de086dfe91fb2007b6c51b06056c579d61234aba))
- **`clipboard_button`**: One-click copy to system clipboard. ([de086df](https://github.com/elcoosp/quoin/commit/de086dfe91fb2007b6c51b06056c579d61234aba))
- **`input`**: Two-way bound text inputs with automatic signal synchronization (GPUI). ([c51efd5](https://github.com/elcoosp/quoin/commit/c51efd5890688eebbae7a7d6c8d4e04496696049))

#### Conformance Test Suite
- **`quoin-conformance`**: Shared test harness verifying all adapters implement `ReactiveContext` correctly. Supports both sync and async (GPUI) test styles. ([da7948e](https://github.com/elcoosp/quoin/commit/da7948e760e357a4788c5e32542cf80606e880e8), [def3f6e](https://github.com/elcoosp/quoin/commit/def3f6e58b6dbab876eb7fbefc14d6cfa66a610f))
- **`provide_global` / `use_global` round-trip tests**: Ensure global state works across all frameworks. ([911e4dd](https://github.com/elcoosp/quoin/commit/911e4dd3be95703aeed00e1c2c0e6e536373d1ef))

#### Macro UI Tests
- **`quoin-macros-tests`**: Comprehensive `trybuild` tests for macro expansion across GPUI, Leptos, and Dioxus. ([62107a8](https://github.com/elcoosp/quoin/commit/62107a89eec309a3885fea715954390f6f26c75a), [18386b7](https://github.com/elcoosp/quoin/commit/18386b7321b30e3ee115d9a87d65c207b29658e2))

#### Examples
- **`counter`**: Framework-agnostic counter hook library. ([d22dee4](https://github.com/elcoosp/quoin/commit/d22dee4e3fdab5aed6cedbbc690548c1901a98fb))
- **Framework-specific counters**: `counter-gpui`, `counter-leptos`, `counter-dioxus`, `counter-floem`, `counter-xilem` demonstrate the same counter hook across all five frameworks. ([ba5c6fe](https://github.com/elcoosp/quoin/commit/ba5c6fe89307b8dceba64e7c52849037df89bdae))
- **`ucp-lib`**: Shared UCP component library demonstrating `component!` and `quoin_render!`. ([08383e9](https://github.com/elcoosp/quoin/commit/08383e97ecd50b1560f35350079f1b7612b87937))
- **UCP demos**: `ucp-demo-gpui`, `ucp-demo-leptos`, `ucp-demo-dioxus` showcase UCP components with reactive state. ([9699fa1](https://github.com/elcoosp/quoin/commit/9699fa1619959653f29181b7a00982c3052e634a), [73b7684](https://github.com/elcoosp/quoin/commit/73b768405fd1e86bfc9ce3b7118f33bde938c750), [6ce8e74](https://github.com/elcoosp/quoin/commit/6ce8e744e4aeb022b88fee1a2b2d45087ec9838d))
- **`mini-devtools-gpui`**: Advanced example with tabs, virtual lists, data tables, and two-way input binding. ([474bc60](https://github.com/elcoosp/quoin/commit/474bc60d47b4ae2420a4cb53528416d2d3d744a0))

#### Documentation
- Comprehensive crate-level documentation for all public APIs. ([03667f0](https://github.com/elcoosp/quoin/commit/03667f0100f693b14e7d431160036cb47caceaeb))
- Enhanced `README.md` with quick start guide, macro examples, and UCP overview. ([62e07c7](https://github.com/elcoosp/quoin/commit/62e07c7a4028eb2f7791fc3c8fc61171b16171ac))

### 🔧 Fixed – Bug fixes and corrections

- Fixed conformance tests for Leptos and Dioxus adapters. ([def3f6e](https://github.com/elcoosp/quoin/commit/def3f6e58b6dbab876eb7fbefc14d6cfa66a610f))
- Dioxus signal updates now correctly propagate to the UI. ([b76cd35](https://github.com/elcoosp/quoin/commit/b76cd35697c2daba2d10927f6db8d36ada25d0ae))
- GPUI counter example: Auto-notify signal updates trigger repaints correctly. ([0bb0fb9](https://github.com/elcoosp/quoin/commit/0bb0fb9f4b786956177a48f87190ab0f62e634fc))
- Dioxus counter example now compiles and runs. ([0d12bac](https://github.com/elcoosp/quoin/commit/0d12bac7884da3f174665af9633dccf471fb91bd))
- Xilem conformance test harness: Added missing runtime. ([c0331ea](https://github.com/elcoosp/quoin/commit/c0331ea89dc39b580a81353128cfbcdddb6199e2))
- Macro parser: Correctly distinguish elements from plain expressions. ([11d8daf](https://github.com/elcoosp/quoin/commit/11d8daf3b9f31893f225daf39a535a3134c82f26))
- `on_click` closures in GPUI now wrapped in `cx.listener` correctly. ([1aa0aa2](https://github.com/elcoosp/quoin/commit/1aa0aa2c133372af492db0dfe3db9f71b5ef914c))
- Removed unnecessary `_ctx` field from GPUI components. ([954ad3d](https://github.com/elcoosp/quoin/commit/954ad3d9a29003e7b62bbffbd1367415a869b9e0))
- UCP demo working across GPUI, Leptos, and Dioxus. ([73b7684](https://github.com/elcoosp/quoin/commit/73b768405fd1e86bfc9ce3b7118f33bde938c750), [6ce8e74](https://github.com/elcoosp/quoin/commit/6ce8e744e4aeb022b88fee1a2b2d45087ec9838d))
- Tailwind styling applied correctly in Dioxus UCP demo. ([6ff1cc5](https://github.com/elcoosp/quoin/commit/6ff1cc511f15caacf0affe253b35e9e94127eb80))
- `effect!` macro UI tests passing for all frameworks. ([34bf602](https://github.com/elcoosp/quoin/commit/34bf602b2177cdb93ec75e39d2667794f62ff3de))
- Trybuild macro tests for Leptos and Dioxus. ([97b505e](https://github.com/elcoosp/quoin/commit/97b505e0b31855fc76a52090456f98256db30ca7), [10cf318](https://github.com/elcoosp/quoin/commit/10cf318a9005262aa696e4472818fe836c68faf1))
- Removed stray dot before `wrapper_styles` in GPUI input rendering. ([1b03094](https://github.com/elcoosp/quoin/commit/1b030948aa91b8076b04f247bf712cec002fee63))
- All examples build and run without errors. ([6bab288](https://github.com/elcoosp/quoin/commit/6bab28880c6d4a4ed0c3282b3af789a1755e6af1))

### ⚡ Performance – Optimizations and improvements

- Switched to `gpui-component` from its git repository for better compatibility. ([f15e101](https://github.com/elcoosp/quoin/commit/f15e101dfc9d005eaa87fcd65695555e40acd7bf))

### 🧹 Cleanup – Code maintenance and housekeeping

- Moved macro tests and core macro logic to separate crates (`quoin-macros-tests`, `quoin-macros-core`). ([18386b7](https://github.com/elcoosp/quoin/commit/18386b7321b30e3ee115d9a87d65c207b29658e2))
- Applied `cargo fmt` and fixed Clippy warnings throughout the workspace. ([f2509f7](https://github.com/elcoosp/quoin/commit/f2509f7c0aa0c393e7734eb519818820cd7b9e9b), [ce96175](https://github.com/elcoosp/quoin/commit/ce96175eb7fe35e7b264ebabf3b993e318d219b2))
- Removed duplicate workspace member entries. ([7ccc886](https://github.com/elcoosp/quoin/commit/7ccc886f7462061545cfe6c3ef28709ebc71a6b5))
- Added `.gitignore` rule for `node_modules`. ([ea75627](https://github.com/elcoosp/quoin/commit/ea75627090e11aa3bccef4a2f33a71742028c6d8))
- Updated to Rust 2024 edition across all crates. ([c18acfb](https://github.com/elcoosp/quoin/commit/c18acfb6f58778b342241de1db66e8fc0114ca57))
