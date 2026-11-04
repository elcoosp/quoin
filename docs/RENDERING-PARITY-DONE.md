# Rendering parity — done

## Outcome

The rendering-parity job is complete: every element on the v1 path lowers to
a native view on both dynamic backends, and every deliberate gap is a
**named, permanent, documented contract** rather than a deferred TODO. On
**Floem**, the native surface covers the full layout/text set (`div`/`h1`–
`h3`/`p`/`span` as `v_stack`/`h_stack`/`label`), `button`, bound `input`,
`checkbox`/`switch` (two-way signal bridges over `Checkbox::new_rw` /
`ToggleButton`), `slider`, `progress`, `badge`, the three `skeleton`s,
`separator`, `scroll_area` (`Scroll`), `virtual_list`, `tabs` (`tab` strip +
`dyn_container` content), `data_table`, `dropdown_menu` (inline items,
matching GPUI's ghost-menu fallback), `clipboard_button`, `tooltip`, keyed
`for` (`dyn_stack` diffing), unkeyed `for` over pure-text bodies
(`dyn_container`), `if`/`else` (one `into_any` seam), `component!`/`run_app!`,
`effect!` (with `cleanup:` running before each re-run and on scope disposal
via `quoin::floem_effect::CleanupSlot`), and `keys { .. }` (macro-time
GPUI-grammar keystroke parsing, exact-modifier matching, focusable focus-
grabbing root, one `KeyDown` listener through `quoin::floem_keys`, shared
`quoin::ev::Key` payloads). On **Xilem**, the native surface covers layout/
text (`flex`/`label`, compile-time text-style cascading), `button`,
`text_input`, `checkbox`/`switch`/`slider`, `progress`, `badge`, skeletons,
`separator`, `portal`-based `scroll_area`, `virtual_scroll`-based
`virtual_list`, `tabs` (button strip + boxed pane), `data_table`,
`dropdown_menu` (inline items), keyed/unkeyed `for`, `if`/`else` (one
`boxed()` seam), and — new in this push — real `component!`/`run_app!`
lowerings (state struct + `fn new(ctx)` + `fn app_logic()` beside the
rebuild pump, booted through `Xilem::new_simple_with_tokio` sharing the
context's tokio runtime). Deliberately unsupported, each with its reason in
`docs/BACKEND-PARITY.md` and enforced both ways by the machine-checked
`emit::parity::BACKEND_PARITY` table: Xilem rejects `tooltip` (masonry has
no tooltip widget), `clipboard_button` (no clipboard API on the branch),
`effect!` (no reactive graph — the app logic is the effect; the pump turns
signal writes into rebuilds) and `keys { .. }` (no global key routing in
v1); both dynamic backends reject `icon`, `img`, `svg`, `rich_text`,
`styled_text`, `radio`, and `radio_group` for the documented contract
reasons (GPUI-specific icon tables, elements outside the v1 cross-backend
contract, single-style v1 text, and the per-item `checked:` contract being
unable to express group exclusivity). All gates are green on a clean
checkout without GTK or xkbcommon dev packages: the expansion gate
(Floem + Xilem + the parity matrix, including the new
`floem_keys_expansion` checks and the stateful-body rule for unkeyed
loops), the allocation gate on **both** backends (the Floem gate takes the
minimum over repeated constructions because floem's reactive runtime
defers effect work into later measurement windows), the `quoin-style`
semantic golden, and the per-adapter conformance suites (85 Floem-side,
40 Xilem-side); `counter-floem`, `todo-floem` (rebuilt on
`component!` + `run_app!` with `keys { "escape" => … }`) and
`counter-xilem` (rebuilt on `component!` + `run_app!`, with the
devtools-instrumented twin kept as a `traced` bin) all build, and CI's
`gates-floem-xilem` job now builds them by manifest path plus a soft
cargo-bloat report that fails only on a >10% regression. En route the
cross-checks flushed out real contract drift that is now fixed: the
Floem/Xilem `virtual_list` emitters read `count:`/`item_height:` although
the parser enforces `items:`/`estimated_height:` (virtual lists always
rendered empty), the tabs emitters emitted `match` arms without commas (a
parse error whenever `tab` children exist), `svg` was missing from the
known-element table so its diagnostic was unreachable, the GPUI `icon`
emitter read `name:` where the parser enforces `icon_name:`, the capture
analysis hoisted nested-closure parameters out of `format!` arguments into
nonexistent scopes, the unkeyed-for diagnostic fired even for pure-text
bodies, and the `data_table` `id:` breaking change is reverted (a missing
`id:` now derives a deterministic per-expansion-site persistence key).
Out of scope for a future milestone: portal-overlay dropdown menus, row-keyed
`data_table` diffing (needs a row key in the element contract), Xilem key
routing and clipboard access, and the `icon`/`img`/`svg`/rich-text elements
pending a cross-backend contract. The honest caveat: Floem and Xilem are
consumed from their **pre-1.0 `main` branches**, not stable releases — the
adapters pin behavior (workspace `[patch.crates-io]` for `muda`, the
generated-under-`target/` xkbcommon discovery for the GPUI trybuild suite),
and the gates exist to catch upstream drift loudly. The changelog
regenerates from the conventional-commit history with `just changelog`
(git-cliff); no public API changed silently — `component!`/`run_app!` on
Xilem and `keys`/`effect!`-cleanup on Floem replace documented diagnostics,
and the `data_table` `id:` requirement never shipped beyond one commit.
