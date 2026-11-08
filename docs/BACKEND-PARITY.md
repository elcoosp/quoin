# Backend parity

What every `quoin_render!` element lowers to on each backend. The **Floem**
and **Xilem** columns are machine-checked: `parity_matrix.rs` in
`quoin-macros-core` cross-checks the emitters against
`quoin_macros_core::emit::parity::BACKEND_PARITY` (the shared const this
document renders), in both directions — a row claiming native must expand
without a diagnostic, and every emitter diagnostic must have an
`unsupported` row here. The **GPUI / Leptos / Dioxus** columns are the
pre-existing reference backends and are documented, not machine-verified:
GPUI notably lowers unknown tags to plain styled `div`s (its shadcn
component layer covers the interactive widgets), and the web backends pass
HTML-ish tags through to the DOM.

Status vocabulary:

- `native` — lowers to a native view of that framework; shared-contract
  semantics.
- `partial` — lowers to a native view with an accepted, documented
  difference (noted inline).
- `unsupported` — emits a named `compile_error!` at the element site. Every
  such entry is a **permanent contract** with its reason, not a deferred
  TODO.

## Element matrix

| element           | floem                                           | xilem                                           | gpui                 | leptos / dioxus        |
| ----------------- | ----------------------------------------------- | ----------------------------------------------- | -------------------- | ---------------------- |
| button            | `floem: native`                                 | `xilem: native`                                 | native               | native                 |
| input             | `floem: native` (bound `text_input`)            | `xilem: native` (`text_input`)                  | native               | native                 |
| checkbox          | `floem: native` (`Checkbox::new_rw` + bridge)   | `xilem: native`                                 | native               | native                 |
| switch            | `floem: native` (`ToggleButton` + bridge)       | `xilem: native`                                 | native               | native                 |
| slider            | `floem: native`                                 | `xilem: native`                                 | native               | native                 |
| progress          | `floem: partial` — rendered as a disabled slider bar | `xilem: partial` — rendered as a disabled slider bar | native | native                 |
| badge             | `floem: native` (rounded container)             | `xilem: native` (sized box)                     | native               | native                 |
| skeleton          | `floem: native`                                 | `xilem: native`                                 | native               | native                 |
| skeleton_text     | `floem: native`                                 | `xilem: native`                                 | native               | native                 |
| skeleton_avatar   | `floem: native`                                 | `xilem: native`                                 | native               | native                 |
| separator         | `floem: native` (1px rule)                      | `xilem: native` (1px rule)                      | native               | native                 |
| scroll_area       | `floem: native` (`Scroll`)                      | `xilem: native` (`portal`)                      | native               | native                 |
| virtual_list      | `floem: native` (`virtual_list`)                | `xilem: native` (`virtual_scroll`)              | native               | native                 |
| tabs              | `floem: native` (`tab` strip + `dyn_container`) | `xilem: native` (button strip + boxed pane)     | native               | native                 |
| data_table        | `floem: partial` — rows rebuild through a `dyn_container`; no row-keyed diffing (the element contract has no row key) | `xilem: partial` — rows rebuild as a fresh Vec sequence each render | native (`gpui_component::DataTable`) | native (HTML table) |
| dropdown_menu     | `floem: partial` — inline item column (matching the GPUI ghost-menu fallback); no portal overlay | `xilem: partial` — inline item column; no portal overlay | native (ghost menu) | native (details/summary or shadcn) |
| clipboard_button  | `floem: native` (`floem::Clipboard`)            | `xilem: unsupported` — xilem/masonry exposes no clipboard API on the v1 branch | native | native (`navigator.clipboard`) |
| tooltip           | `floem: native` (`tooltip` view)                | `xilem: unsupported` — masonry has no tooltip widget; wrapping in a hint container changes semantics, so v1 rejects it | native | native (`title` attr) |
| icon              | `floem: unsupported` — the icon-name table is gpui-component-specific; pass an explicit `svg`/styled label instead | `xilem: unsupported` — the icon-name table is gpui-component-specific; pass an explicit `svg`/styled label instead | native (`IconName` table) | native (icon font) |
| img               | `floem: unsupported` — not in the v1 cross-backend element contract (only the web backends pass `img` through as HTML) | `xilem: unsupported` — not in the v1 cross-backend element contract (only the web backends pass `img` through as HTML) | unsupported (use `icon`) | native (HTML `img`) |
| svg               | `floem: unsupported` — not in the v1 cross-backend element contract; use a styled label or a backend-specific view | `xilem: unsupported` — not in the v1 cross-backend element contract; use a styled label or a backend-specific view | unsupported | native (inline HTML `svg`) |
| rich_text         | `floem: unsupported` — not on the v1 path; the shared text contract is single-style text | `xilem: unsupported` — not on the v1 path; the shared text contract is single-style text | unsupported | native (HTML spans) |
| styled_text       | `floem: unsupported` — not on the v1 path; the shared text contract is single-style text | `xilem: unsupported` — not on the v1 path; the shared text contract is single-style text | native (`StyledRun`s) | native (HTML spans) |
| radio             | `floem: unsupported` — the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox` | `xilem: unsupported` — the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox` | unsupported | native (HTML `radio`) |
| radio_group       | `floem: unsupported` — the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox` | `xilem: unsupported` — the v1 per-item `checked:` contract cannot express group exclusivity; use `checkbox` | unsupported | native (HTML `radio`) |

Plain containers (`div`, `h1`–`h3`, `p`, `span`, `label`, `a`, `ul`, `ol`,
`li`, `hr`, `br`, `textarea`, `select`, `form`) lower to the native layout
primitives on both dynamic backends (`v_stack`/`h_stack`/`label` on Floem,
`flex`/`label` on Xilem). Structural children (`tab`, `column`, `item`) only
appear inside their parent element and have no standalone lowering.

## Non-element contracts

| construct            | floem                                            | xilem                                             |
| -------------------- | ------------------------------------------------ | ------------------------------------------------- |
| `if[c] { .. }`       | native — `dyn_container`, exactly one boxing seam | native — branch-selected view, one `boxed()` seam |
| `for[item in list, key: k]` | native — diffing `dyn_stack`                | native — index-keyed `Vec` sequence               |
| `for[item in list]` (unkeyed) | native for pure-text bodies (`dyn_container` full rebuild); named diagnostic when the body captures state or contains interactive elements | native |
| `keys { .. }`        | native — focusable root + one `KeyDown` listener through `quoin::floem_keys`; exact-modifier, GPUI-grammar keystrokes | `xilem: unsupported` — no global key routing in v1; handle input on the specific widget |
| `effect! { .. }`     | native — `Effect::new`; `cleanup:` runs before each re-run and when the owning reactive scope is disposed (`quoin::floem_effect::CleanupSlot`) | `xilem: native` — `quoin::xilem_effect::create_effect` over the core reactive graph: the effect observes the component's `XilemSignal`s through their dependency cells, re-runs FIFO and batch-coalesced; `cleanup:` runs before each re-run and a final time when the effect is disposed with its component scope (`quoin::xilem_effect::CleanupSlot`). Effects belong at component scope (`on_mount { .. }`, where `component!` holds the component's `EffectScope`); `effect!` in the render body is a named compile error — `app_logic` re-runs per rebuild, so the effect would be re-created every time |
| `component!`         | native — a plain `fn` creating signals once and returning the view | native — state struct + `fn new(ctx)` + `fn app_logic()` rendering beside the rebuild pump |
| `run_app!`           | native — `floem::launch`                          | native — `Xilem::new_simple_with_tokio` + pump notifier, sharing the context's tokio runtime |

## Known allowed differences

- Xilem has no style inheritance: text decls on a container are compiled
  (cascaded) onto every text leaf under it, and many container-only decls
  are accepted no-ops there. Changing one of those no-ops requires updating
  this document and the `quoin-style` semantic golden in the same commit.
- The `data_table` contract carries no row key, so Floem rebuilds rows
  wholesale; append/remove/reorder stay correct, only identity is lost.
- `dropdown_menu` renders inline on both dynamic backends (and GPUI's ghost
  menu behaves the same); a portal overlay is future work on all three.
- Floem's `keys` support maps `cmd`/`super`/`meta` to the platform meta
  modifier and reports it as `ctrl` on the shared `quoin::ev::Key` payload
  (the shared type has a single platform-modifier flag).

## Pre-1.0 caveat

Floem and Xilem are consumed from their **`main` branches** (pre-1.0, git
dependencies), not stable releases: `floem` at
`1351ffb162faeb8be983f1301d718a0d5fb59c27` and `xilem` at its current
`main`. APIs used here (`SyncRwSignal`, `worker`/`MessageProxy`,
`ValueContainer`, Masonry properties) may shift upstream; the adapters and
emitters pin behavior, and the gates exist to catch drift loudly.
