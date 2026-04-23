# quoin — Phase 2 Brainstorm: Full Leptos & Dioxus Emission Parity

> **When:** after the v2 roadmap is complete.  
> **Goal:** `quoin_render!` on Leptos and Dioxus produces output that is visually and functionally indistinguishable from the GPUI version, using the shadcn component ecosystem as the backing library for each web framework.

---

## The landscape (researched April 2026)

### Leptos → `cloud-shuttle/leptos-shadcn-ui`
38 components published on crates.io at v0.7–0.9, covering the full shadcn canon:  
Button, Input, Label, Checkbox, Switch, Radio Group, Select, Textarea, Card, Separator, **Tabs**, Accordion, **Dialog**, Popover, **Tooltip**, Alert, **Badge**, Skeleton, Progress, Toast, **Table**, Calendar, Date Picker, Pagination, Breadcrumb, and more. Notably **VirtualList / ScrollArea / DropdownMenu** are present in the sister `radix-leptos` crate from the same org.

Missing / uncertain from published list: **VirtualList** (windowing), **DropdownMenu** with checked items (needed for the event filter), **StyledText** runs (timeline search highlight), **Clipboard** API, **Resizable** table columns.

### Dioxus → `MBeliou/shadcn-dioxus`
A smaller, actively developed shadcn port (8 stars, 148 commits as of research date). Component coverage is narrower — Button, Badge, Card, Dialog, Input, Select, Tabs, Tooltip confirmed. **Table, VirtualList, DropdownMenu with checked items** are absent or incomplete. DioxusLabs itself is building first-party primitives (`DioxusLabs/components`) that landed with Dioxus 0.7, giving a solid accessibility foundation.

### GPUI → `gpui-component` (via `quoin-ui-gpui`)
The reference implementation. Has everything the Devtools uses: `v_virtual_list`, `DataTable` with `TableDelegate`, `TabBar`/`Tab`, `DropdownMenu` with `PopupMenuItem::checked()`, `Input`/`InputState`, `Clipboard`, `StyledText` with run-level styling, `Icon`, `Button` with variants.

### What this tells us
The web frameworks have *most* of the primitives but the mapping is not 1-to-1:

| quoin element | GPUI | Leptos | Dioxus |
|---|---|---|---|
| `button` | `Button` | `leptos-shadcn-button` ✅ | `shadcn-dioxus` Button ✅ |
| `input` | `Input`+`InputState` | `leptos-shadcn-input` ✅ | `shadcn-dioxus` Input ✅ |
| `tabs` / `tab` | `TabBar`+`Tab` | `leptos-shadcn-tabs` ✅ | `shadcn-dioxus` Tabs ✅ |
| `badge` | `Badge` (icon render) | `leptos-shadcn-badge` ✅ | `shadcn-dioxus` Badge ✅ |
| `dropdown_menu` | `DropdownMenu`+`PopupMenuItem` | `radix-leptos` DropdownMenu ⚠️ checked items? | absent from MBeliou ❌ → use DioxusLabs/components |
| `data_table` | `DataTable`+`TableDelegate` | `leptos-shadcn-table` ✅ basic | absent from MBeliou ❌ → build with `<table>` HTML |
| `virtual_list` | `v_virtual_list` | `radix-leptos` VirtualList ⚠️ | `dioxus-primitives` RecycleList ⚠️ |
| `clipboard_button` | `Clipboard` | `web_sys` clipboard API ✅ | `web_sys` clipboard API ✅ |
| `StyledText` runs | `StyledText::with_runs` | `<span>` with inline style ✅ | `span` with style ✅ |
| `rich_text` | `gpui_component::StyledText` | CSS spans | CSS spans |
| `tooltip` | built into Button `.tooltip()` | `leptos-shadcn-tooltip` ✅ | `shadcn-dioxus` Tooltip ✅ |
| `icon` | `Icon::new(IconName::X)` | lucide-leptos / SVG ⚠️ | lucide-dioxus / SVG ⚠️ |

Legend: ✅ available and mapped, ⚠️ exists but needs custom wiring, ❌ missing — need to build or fallback.

---

## The core problem: two-way input binding on the web

On GPUI, `InputState` is a persistent entity owned by the component struct. On Leptos/Dioxus the DOM owns the input state and the framework's signal system bridges it. The `quoin_render!` input emitter currently stubs this as a plain `<input>` tag. For full parity it needs:

- **Leptos:** `<input value=move || signal.get() on:input=move |ev| signal.set(event_target_value(&ev)) />`
- **Dioxus:** `input { value: "{signal}", oninput: move |ev| signal.set(ev.value()), }`

This is actually *simpler* than the GPUI path — no `InputState` entity needed. The macro just needs to emit the right binding syntax per framework.

---

## Phase 2-A  Dependency wiring — `Cargo.toml` additions

### `quoin-leptos/Cargo.toml`
```toml
leptos-shadcn-button  = { version = "0.4", optional = true }
leptos-shadcn-input   = { version = "0.4", optional = true }
leptos-shadcn-tabs    = { version = "0.4", optional = true }
leptos-shadcn-badge   = { version = "0.4", optional = true }
leptos-shadcn-table   = { version = "0.4", optional = true }
leptos-shadcn-tooltip = { version = "0.4", optional = true }
leptos-shadcn-dialog  = { version = "0.4", optional = true }

[features]
shadcn = [
    "dep:leptos-shadcn-button", "dep:leptos-shadcn-input",
    "dep:leptos-shadcn-tabs",   "dep:leptos-shadcn-badge",
    "dep:leptos-shadcn-table",  "dep:leptos-shadcn-tooltip",
    "dep:leptos-shadcn-dialog",
]
```

### `quoin-dioxus/Cargo.toml`
```toml
shadcn-dioxus = { git = "https://github.com/MBeliou/shadcn-dioxus", optional = true }

[features]
shadcn = ["dep:shadcn-dioxus"]
```

### `quoin-macros-core/Cargo.toml`
```toml
[features]
leptos-shadcn = []   # gates the enhanced Leptos emitter
dioxus-shadcn = []   # gates the enhanced Dioxus emitter
```

The `shadcn` feature on the adapter crates flows through `quoin` → `quoin-macros` → `quoin-macros-core` via feature unification, letting the emitter know which component library to target.

---

## Phase 2-B  Enhanced input two-way binding (Leptos + Dioxus)

Replace the current bare `<input>` stub in `emit/render_leptos.rs` and `emit/render_dioxus.rs` with proper reactive binding:

**Leptos:**
```rust
fn emit_input(el: &Element) -> TokenStream {
    let placeholder = find_arg_string(el, "placeholder").unwrap_or_default();
    if let Some(value_expr) = find_arg_expr(el, "value") {
        // full two-way binding
        quote! {
            <input
                class={#class_expr}
                placeholder={#placeholder}
                prop:value={move || #value_expr.get()}
                on:input={move |ev| #value_expr.set(leptos::ev::event_target_value(&ev))}
            />
        }
    } else {
        quote! { <input class={#class_expr} placeholder={#placeholder} /> }
    }
}
```

**Dioxus:**
```rust
quote! {
    input {
        class: {#class_expr},
        placeholder: {#placeholder},
        value: "{#value_expr}",
        oninput: move |ev| #value_expr.set(ev.value()),
    }
}
```

When `shadcn` feature is active, wrap in the library's `Input` component instead of a bare `<input>`.

---

## Phase 2-C  `tabs` — shadcn wiring

**Leptos (leptos-shadcn-tabs):**
```rust
// Current: div {} stub
// After:
quote! {
    <leptos_shadcn_tabs::Tabs
        default_value={#active_expr.to_string()}
        on_value_change={move |val: String| {
            if let Ok(idx) = val.parse::<usize>() { #on_click_expr(idx) }
        }}
    >
        <leptos_shadcn_tabs::TabsList>
            #(#tab_triggers)*
        </leptos_shadcn_tabs::TabsList>
    </leptos_shadcn_tabs::Tabs>
}
```

**Dioxus (shadcn-dioxus Tabs):**
```rust
quote! {
    shadcn_dioxus::tabs::Tabs {
        value: "{#active_expr}",
        on_value_change: move |val: String| { /* parse + call callback */ },
        shadcn_dioxus::tabs::TabsList {
            #(#tab_triggers)*
        }
    }
}
```

When `shadcn` feature is off (plain web output), fall back to the existing `div`-based tabs.

---

## Phase 2-D  `dropdown_menu` — shadcn wiring

**Leptos** — use `radix-leptos` DropdownMenu (from `cloud-shuttle/radix-leptos`):
```rust
quote! {
    <radix_leptos::dropdown_menu::DropdownMenuRoot>
        <radix_leptos::dropdown_menu::DropdownMenuTrigger>
            {#trigger_expr}
        </radix_leptos::dropdown_menu::DropdownMenuTrigger>
        <radix_leptos::dropdown_menu::DropdownMenuContent>
            #(#item_tokens)*
        </radix_leptos::dropdown_menu::DropdownMenuContent>
    </radix_leptos::dropdown_menu::DropdownMenuRoot>
}
```

Each `item(label: …, on_click: …)` child maps to `<DropdownMenuItem on:click=…>label</DropdownMenuItem>`.  
Checked items: `<DropdownMenuCheckboxItem checked={#is_checked} on_checked_change={…}>`.

**Dioxus** — MBeliou doesn't have this. Use DioxusLabs/components `DropdownMenu` primitive, or implement a plain popover pattern with `use_signal(bool)` for open state:
```rust
quote! {
    {
        let __open = use_signal(|| false);
        div { class: "relative",
            div { onclick: move |_| __open.toggle(),
                {#trigger_expr}
            }
            if *__open.read() {
                div { class: "absolute z-50 min-w-32 rounded-md border bg-popover shadow-md",
                    #(#item_tokens)*
                }
            }
        }
    }
}
```

When DioxusLabs/components adds a stable DropdownMenu primitive, upgrade the emitter.

---

## Phase 2-E  `virtual_list` — DOM virtualization

The GPUI virtual list is a retained-mode `v_virtual_list` — no direct DOM equivalent. Web frameworks use JS-style windowing.

**Leptos** — use `radix-leptos` VirtualList or a simple CSS-based overflow container (good enough for the devtools use case where lists are bounded):
```rust
// Option A: simple scrolling div (no true windowing — acceptable for <1000 items)
quote! {
    <div class="overflow-y-auto h-full">
        <leptos::prelude::For
            each=move || #items_expr.clone()
            key=|item| item.id
            children=move |item| leptos::view! { #item_template }
        />
    </div>
}

// Option B: leptos-virtual-list crate if item count is large
quote! {
    <leptos_virtual_list::VirtualList
        items=move || #items_expr.clone()
        item_height=#estimated_height
        key=|item| item.id
    >
        {move |item| leptos::view! { #item_template }}
    </leptos_virtual_list::VirtualList>
}
```

The emitter chooses Option A when no `estimated_height` is given, Option B when it is, and records a `TODO: upgrade to windowing` comment if `estimated_height` is given but `leptos-virtual-list` is not in scope.

**Dioxus** — `dioxus-primitives` ships a `RecycleList` (released with Dioxus 0.7 primitives). Map to it directly:
```rust
quote! {
    dioxus_primitives::RecycleList {
        items: #items_expr.clone(),
        item_height: #estimated_height,
        render: move |item| rsx! { #item_template }
    }
}
```

---

## Phase 2-F  `data_table` — full Leptos/Dioxus parity

**Leptos** — `leptos-shadcn-table` provides a basic `<Table>` but not a full delegate with sorting. Build a thin wrapper that compiles to:
```rust
quote! {
    <leptos_shadcn_table::Table>
        <leptos_shadcn_table::TableHeader>
            <leptos_shadcn_table::TableRow>
                #(#header_cells)*     // with sort chevron if sortable: true
            </leptos_shadcn_table::TableRow>
        </leptos_shadcn_table::TableHeader>
        <leptos_shadcn_table::TableBody>
            <leptos::prelude::For
                each=move || {
                    let mut rows = #rows_expr.clone();
                    if let Some((col, dir)) = __sort_state.get() {
                        rows.sort_by(/* use sort_key closures */);
                    }
                    rows
                }
                key=|row| row.id
                children=move |row| view! {
                    <leptos_shadcn_table::TableRow>
                        #(#row_cells)*
                    </leptos_shadcn_table::TableRow>
                }
            />
        </leptos_shadcn_table::TableBody>
    </leptos_shadcn_table::Table>
}
```

Local `__sort_state: Signal<Option<(String, SortDir)>>` is injected by the emitter.

**Dioxus** — no table library in MBeliou; emit a plain HTML table with Tailwind:
```rust
quote! {
    table { class: "w-full text-sm",
        thead {
            tr { #(#header_cells)* }
        }
        tbody {
            #rows_expr.iter().map(|row| rsx! {
                tr { #(#row_cells)* }
            })
        }
    }
}
```

---

## Phase 2-G  `StyledText` / search highlight — web equivalent

GPUI has `StyledText::with_runs(vec![TextStyle { color, background_color, … }.to_run(len)])`.

On the web this is just nested `<span>` elements with inline styles or CSS classes. The emitter should generate a helper function at the call site:

**Leptos:**
```rust
// Generated helper (injected once per component)
fn __highlight_text(text: &str, query: &str) -> impl IntoView {
    if query.is_empty() { return view! { {text.to_string()} }; }
    // split text at query positions, wrap matches in <mark>
    let parts = split_with_highlight(text, query);
    parts.into_iter().map(|(chunk, is_match)| view! {
        <span class={if is_match { "bg-yellow-300 text-black" } else { "" }}>
            {chunk}
        </span>
    }).collect_view()
}
```

The `virtual_list` item template uses this helper automatically when the `search_query` binding is present.

**Dioxus:** same pattern, `rsx!` wrapping.

---

## Phase 2-H  `badge` element — new quoin_render! element

The Devtools uses colored badge labels (`"NAV"`, `"BLD"`, etc.) extensively. Add `badge` as a first-class element:

```rust
badge(text: display.badge, color: display.badge_color)
```

**GPUI:** emits `div().px_1().rounded(px(2.0)).bg(color).text_color(white()).child(text)`.

**Leptos:** emits `<leptos_shadcn_badge::Badge class="…">{text}</leptos_shadcn_badge::Badge>`.

**Dioxus:** emits `shadcn_dioxus::badge::Badge { class: "…", "{text}" }`.

The `color` arg maps from a `Hsla` (GPUI) to a CSS class string (web). Add a `color_to_class` mapping in the Tailwind transpiler.

---

## Phase 2-I  `icon` element — cross-framework icon mapping

**GPUI:** `Icon::new(IconName::Calendar)` — already works.

**Leptos:** use `lucide-leptos` crate:
```rust
// icon(name: "calendar")
quote! { <lucide_leptos::Calendar class="w-4 h-4" /> }
```

**Dioxus:** use `lucide-dioxus` crate:
```rust
quote! { lucide_dioxus::Calendar { class: "w-4 h-4" } }
```

Add `"icon"` to `KNOWN_ELEMENTS`. The `name` arg maps to the correct import via a match table in `render_leptos.rs`/`render_dioxus.rs`. Start with the ~15 icons used in the Devtools: Calendar, Info, Inbox, Folder, Settings, Search, Close, Trash, File, Play, Map, Copy, ChevronRight, ChevronDown, Loader.

---

## Phase 2-J  `tooltip` attribute on `button`

GPUI allows `.tooltip("text")` chained on a Button. On the web, wrap the button in a Tooltip component:

```rust
// In render_leptos.rs, when button has tooltip arg:
let tooltip_text = find_arg_string(el, "tooltip");
if let Some(text) = tooltip_text {
    quote! {
        <leptos_shadcn_tooltip::TooltipProvider>
            <leptos_shadcn_tooltip::Tooltip>
                <leptos_shadcn_tooltip::TooltipTrigger>
                    #button_tokens
                </leptos_shadcn_tooltip::TooltipTrigger>
                <leptos_shadcn_tooltip::TooltipContent>
                    {#text}
                </leptos_shadcn_tooltip::TooltipContent>
            </leptos_shadcn_tooltip::Tooltip>
        </leptos_shadcn_tooltip::TooltipProvider>
    }
}
```

Add `"tooltip"` to `KNOWN_ARGS`.

---

## Phase 2-K  Clipboard API

The current `clipboard_button` stub for web says "use `web_sys`". Flesh it out:

```rust
// render_leptos.rs
"clipboard_button" => {
    let copy_text = find_arg_expr(el, "copy_text").expect(…);
    quote! {
        <button
            class={#class_str}
            onclick={move |_| {
                let text = #copy_text.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(clipboard) = web_sys::window()
                        .and_then(|w| w.navigator().clipboard())
                    {
                        let _ = wasm_bindgen_futures::JsFuture::from(
                            clipboard.write_text(&text)
                        ).await;
                    }
                });
            }}
        >
            #(#child_tokens)*
        </button>
    }
}
```

Same pattern for Dioxus, using `dioxus::web::WebEventExt` + `spawn`.

Add `wasm-bindgen-futures` and `web-sys` (with `Clipboard,Window,Navigator` features) to `quoin-leptos` and `quoin-dioxus` dependencies, gated behind the `web` or `shadcn` feature.

---

## Phase 2-L  Theme token mapping — GPUI `Hsla` → CSS variables

GPUI uses `theme.primary`, `theme.success`, `theme.warning`, `theme.muted_foreground` etc. (all `Hsla` values).

The web frameworks use CSS variables from the shadcn theme: `--primary`, `--destructive`, `--muted-foreground`, etc.

Add a `ThemeTokenMapper` in `quoin-macros-core/src/transpile/`:

```rust
// Maps quoin ThemeToken to shadcn CSS variable class
pub fn token_to_tailwind_class(token: &str) -> &'static str {
    match token {
        "primary"          => "text-primary",
        "success"          => "text-green-500",
        "warning"          => "text-yellow-500",
        "info"             => "text-blue-500",
        "muted_foreground" => "text-muted-foreground",
        "foreground"       => "text-foreground",
        "background"       => "bg-background",
        _ => "",
    }
}
```

When the GPUI emitter encounters `text_color(theme.primary)`, the Leptos/Dioxus emitters instead emit `class="text-primary"`. This requires the render emitter to detect `theme.xxx` expressions and substitute the CSS class — add a `ThemeExprTransformer` visitor to `render_leptos.rs`/`render_dioxus.rs`.

---

## Phase 2-M  `scroll_area` / overflow scrolling

GPUI uses `.overflow_x_scrollbar()` / `.overflow_y_scrollbar()` method chaining. On the web this maps to:

- **Leptos:** wrap in `<div class="overflow-x-auto">` or (when `shadcn` feature active) `<ScrollArea class="…">`.
- **Dioxus:** same.

Add `scroll_area` as a quoin element:
```rust
scroll_area(class: "h-full", direction: "vertical") { … }
```

GPUI emitter: emits `.overflow_y_scrollbar()` on a wrapping `div`.  
Leptos/Dioxus: emits the shadcn `ScrollArea` component or plain `overflow-y-auto`.

---

## Phase 2-N  `quoin-macros-core` feature gates for the new emitters

Add `shadcn` sub-feature logic throughout the emitters:

```rust
// emit/render_leptos.rs
fn emit_button(el: &Element) -> TokenStream {
    #[cfg(feature = "leptos-shadcn")]
    return emit_button_shadcn(el);
    #[cfg(not(feature = "leptos-shadcn"))]
    return emit_button_plain(el);
}
```

`emit_button_plain` is the current HTML-based emitter. `emit_button_shadcn` uses `leptos_shadcn_button::Button`.

This means all components have a working fallback even without the shadcn dep — important for users who don't want the full shadcn stack.

---

## Phase 2-O  New example: `ucp-demo-leptos-shadcn`

Add a new workspace-excluded example that demonstrates parity:

```toml
# examples/ucp-demo-leptos-shadcn/Cargo.toml
[dependencies]
quoin = { path = "../../quoin", features = ["leptos", "leptos-shadcn"] }
ucp-lib = { path = "../ucp-lib", features = ["leptos", "leptos-shadcn"] }
leptos-shadcn-ui = { version = "0.9.0", features = ["button", "input", "tabs", "badge", "table", "tooltip"] }
```

The `MiniDevtools` component compiles against this example unchanged — proving the abstraction holds.

---

## Risk register and mitigations

| Risk | Likelihood | Mitigation |
|---|---|---|
| `MBeliou/shadcn-dioxus` is low-maintenance (8 stars) | Medium | Fall back to `DioxusLabs/components` for missing pieces; emit plain HTML for anything not covered |
| `leptos-shadcn-ui` Table/VirtualList is behind locked paths (not on crates.io) | Low | Use `radix-leptos` sister crate from same org, which does publish VirtualList |
| `v_virtual_list` semantics (retained mode) have no 1-to-1 DOM equivalent | High for scroll performance | Accept that web virtualization via CSS `overflow-y-auto` + `<For>` is correct for list sizes in the devtools context (< 500 events). Opt-in to true windowing via `estimated_height` arg |
| Theme token mismatch (`Hsla` vs CSS var) for custom colors | Medium | The `ThemeExprTransformer` only handles known `theme.xxx` patterns; arbitrary `Hsla` values fall back to inline `style="color: hsl(…)"` |
| `leptos-shadcn-tabs` controlled vs uncontrolled API | Low | Controlled mode (passing `value` + `on_value_change`) is the correct pattern; confirm it works in a smoke test before wiring the emitter |

---

## Implementation order

| Task | Unlocks | ~Time |
|---|---|---|
| 2-A Dependency wiring + feature flags | All 2-B to 2-N | 2 h |
| 2-B Input two-way binding (Leptos + Dioxus) | Core UX, search filter | 2 h |
| 2-C Tabs (shadcn wiring) | Tab navigation | 1 h |
| 2-G StyledText / search highlight | Timeline search UX | 1.5 h |
| 2-D DropdownMenu (shadcn wiring) | Filter types button | 2 h |
| 2-E VirtualList (simple + windowed) | Timeline tab | 2 h |
| 2-F DataTable (Leptos shadcn + Dioxus plain) | Cache tab | 3 h |
| 2-H Badge element | Event type labels | 1 h |
| 2-I Icon element | All icon usage | 2 h |
| 2-J Tooltip on button | Invalidate/remove buttons | 1 h |
| 2-K Clipboard API (web) | Copy all / export | 1 h |
| 2-L Theme token mapping | Visual parity | 2 h |
| 2-M scroll_area element | Overflow containers | 1 h |
| 2-N Feature-gated emitters (plain fallback) | Gradual adoption | 2 h |
| 2-O Example: ucp-demo-leptos-shadcn | Smoke test | 1 h |
| **Total** | | **~25 h** |

---

## What full parity looks like

After Phase 2, the same `component! { pub MiniDevtools { … } }` block compiles to:

**GPUI** — `gpui_component` backed, native retained-mode UI, pixel-perfect. **(already works after v2 roadmap)**

**Leptos** — `leptos-shadcn-ui`/`radix-leptos` backed, WASM, full SSR support, accessible, Tailwind-styled, visually indistinguishable. Input two-way binding via signal↔DOM, tabs via shadcn Tabs, virtual list via leptos-virtual-list, dropdown via radix-leptos, table via leptos-shadcn-table.

**Dioxus** — `shadcn-dioxus`/`DioxusLabs components` backed, native desktop + WASM, Tailwind-styled. Gaps filled with plain HTML fallbacks where the library is immature.

All three from one source. The only framework-specific code lives in `quoin-macros-core/src/emit/` — the component author never touches it.
