# The `gpui-rsx-classes` engine — extended Tailwind vocabulary, variants, and dynamic classes on GPUI

Phase 1 + Phase 2 of the gpui-rsx migration plan
(`quoin-gpui-rsx-migration-report.md`, §8): quoin's GPUI backend gains
gpui-rsx's class engine, **enabled by default with the `gpui` feature**
(the standalone `gpui-rsx-classes` flag remains for `macros`-only users),
without changing the output of anything that compiled before.

## TL;DR

```toml
# your crate's Cargo.toml
[dependencies.quoin]
version = "…"
# `gpui` implies `gpui-rsx-classes` — no second flag to remember.
features = ["gpui"]
```

```rust
// Before: compile error — quoin's palette stops at blue-600.
quoin_render! { div(class: "text-blue-400 w-1/2 focus:border-blue-500") { … } }
// With the flag: all three resolve.
```

- **Nothing that compiled before changes.** Classes quoin knows keep their
  byte-identical lowering (theme tokens included — same baked shadcn-dark
  hexes). The engine only decides classes quoin *rejects* today.
- **New vocabulary.** The full Tailwind palette (242 colors), fractions
  (`w-1/2`), arbitrary values (`w-[280px]`, `gap-[14px]`), `border-t-2`,
  grid placement — resolved at expansion time by
  `gpui_rsx_core::analyze_class` in **strict mode** (unknown classes stay
  compile errors with did-you-mean suggestions, preserving quoin's contract;
  negative utilities like `-mt-2` are rejected in strict mode by
  gpui-rsx-core 0.10.2 with its own guidance).
- **New state variants.** `focus:`, `focus-visible:`, `active:`,
  `group-hover:`, `group-active:` (with named groups) — `hover:` was already
  supported and keeps its exact shape. Non-hover variants automatically make
  the element stateful (`.id(..)`), mirroring the scroll-overflow injection.
- **Dynamic classes work.** `class: some_expr` was silently dropped on GPUI;
  under the flag it resolves at render time through an inline `rsx_runtime`
  module (gpui-rsx's generated matcher: static table + color parsing +
  numeric parsing). Unknown dynamic classes panic (strict). Static classes
  only — put variants in static `class:` arguments.
- **`rsx.toml` extends the vocabulary.** Put an `rsx.toml` next to your
  crate's `Cargo.toml` and declare custom tokens; they resolve like any
  class, and the file is tracked so editing it recompiles your crate:

  ```toml
  [colors]
  brand = "#1d4ed8"
  ```

  ```rust
  quoin_render! { div(class: "bg-brand") { … } }
  ```

- **The `rsx!` escape hatch.** The `gpui` feature now re-exports gpui-rsx's
  macros from `quoin` (`rsx!`, `rsx_strict!`, `rsx_permissive!`,
  `rsx_fragment!`, `rsx_tw!`, `rsx_expand!`, `rsx_runtime!`, `#[component]`),
  so GPUI-only subtrees can use gpui-kit components directly:

  ```rust
  // `gpui` here is the app's gpui-kit alias (`gpui = { package = "gpui-kit" }`);
  // extension traits must be in scope at the call site.
  use gpui::component::button::{Button, ButtonVariants as _};

  quoin_render! {
      div(class: "flex flex-col gap-4") {
          quoin::rsx! {
              <Button base={Button::new("save")} label="Save" primary class="rounded-md" />
          }
      }
  }
  ```

  Use `quoin::rsx_tw!` for quoin's 4 px-per-unit spacing scale (plain
  `quoin::rsx!` keeps gpui-rsx's 1 px-per-unit default) — the same switch the
  engine applies for you on the `class:` path.

## How the engine decides (per whitespace-separated class)

| quoin parse | engine outcome | emitted |
| --- | --- | --- |
| known (incl. theme tokens) | *engine not consulted* | existing lowering, byte-identical |
| unknown | `Supported` | gpui-rsx tokens |
| unknown + `focus:`/`active:`/`group-*:` | `Supported` | refinement + stateful element |
| unknown | `Error` / `Ignored` | `compile_error!` with the engine's message + quoin's suggestion |
| non-literal `class:` expr | — | render-time resolution (Phase 3) |

Phase 5 of the migration plan ("flip the default") reverses the first row:
gpui-rsx's engine takes over the known-class lowering too, after one release
of the flag proving byte-parity in real apps.

## Verification layers

1. **`interop-tests`** — the Phase-0 parity harness, committed as a workspace
   crate: 34/34 theme-token classes byte-identical through gpui-rsx's
   `Theme`, the vocabulary-superset gate, the Tailwind scale gate, and the
   variant table (all APIs compiled against gpui-rsx-core 0.10.2).
2. **`quoin-macros-core` unit tests** (`--features gpui-rsx-classes`) — the
   decision policy table above, per class, at token level.
3. **`quoin-grsx-compiletest`** — compiles full `quoin_render!` expansions
   (known + new vocabulary + variants + dynamic + control flow + handlers)
   against the vendored GPUI stub (`third_party/gpui-stub`, from gpui-rsx
   v0.10.2), whose builder surface is generated from the *same*
   `gpui-pre@0.3.10` snapshot quoin pins — proving the emitted chains
   type-check without the multi-minute gpui-pre build.
4. Real-GPUI coverage stays with `quoin-macros-tests --features gpui` and
   `quoin-conformance --features gpui` (enable the flag there as well when
   trying it out).

## Version discipline

`gpui-rsx-core` / `gpui-rsx` pin `0.10.2`, whose tables are generated from
`gpui-pre@0.3.10` — exactly quoin's pin (and gpui-component 0.8.1's). Bump
all of them together; gpui-rsx's `cargo xtask check-gpui` guards the pairing
upstream.
