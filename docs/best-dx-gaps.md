I'm using the writing-plans skill to create the implementation plan.

The plan will be saved to `docs/superpowers/plans/2026-04-20-quoin-ucp-devtools-gap-closure.md`.

---

# Quoin UCP Gap Closure — Navi Devtools Target

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the feature gaps identified by the Navi Devtools component so that a framework‑agnostic version can be written entirely in `quoin_render!` and `component!`, running on GPUI, Leptos, and Dioxus.

**Architecture:** Extend the `quoin` reactive core with `effect!` and `use_global`, build native UCP adapters for Button, Input, DataTable, VirtualList, DropdownMenu, and TabBar, enhance `quoin_render!` with additional Tailwind classes and pseudo‑class support, and add a Clipboard service.

**Tech Stack:** Rust, syn/quote, GPUI, Leptos 0.8, Dioxus 0.7, `gpui-component` (GPUI), `leptos-shadcn-ui` (Leptos), `shadcn-dioxus` (Dioxus).

---

## Chunk 1: Core Reactive Primitives

### Task 1: `effect!` Macro

**Files:**
- Create: `quoin-macros/src/effect.rs`
- Modify: `quoin-macros/src/lib.rs`
- Create: `quoin-macros/tests/ui/effect_pass.rs`

- [ ] **Step 1: Define `Effect` AST**

```rust
// quoin-macros/src/effect.rs
use syn::parse::{Parse, ParseStream};
use syn::{braced, Expr, Ident, Token};

pub struct Effect {
    pub deps: Vec<Ident>,
    pub body: Expr,
}

impl Parse for Effect {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        braced!(content in input);
        content.parse::<Token![watch]>()?;
        content.parse::<Token![:]>()?;
        let deps_content;
        syn::bracketed!(deps_content in content);
        let mut deps = Vec::new();
        while !deps_content.is_empty() {
            deps.push(deps_content.parse::<Ident>()?);
            if deps_content.peek(Token![,]) {
                deps_content.parse::<Token![,]>()?;
            }
        }
        content.parse::<Token![,]>()?;
        let body: Expr = content.parse()?;
        Ok(Effect { deps, body })
    }
}
```

- [ ] **Step 2: Emit GPUI version (observer pattern)**

```rust
// emit/gpui.rs effect emitter
pub fn emit_effect(effect: &Effect) -> TokenStream {
    let deps = &effect.deps;
    let body = &effect.body;
    quote! {{
        use quoin_gpui::Observer;
        let __observer = Observer::new();
        #( __observer.observe(&#deps, move |_| { #body; }); )*
        __observer
    }}
}
```

- [ ] **Step 3: Emit Leptos version (`create_effect`)**

```rust
// emit/leptos.rs effect emitter
quote! {
    leptos::prelude::create_effect(move |_| {
        #body;
    });
}
```

- [ ] **Step 4: Emit Dioxus version (`use_effect`)**

```rust
// emit/dioxus.rs effect emitter
quote! {
    dioxus::prelude::use_effect(move || {
        #body;
    });
}
```

- [ ] **Step 5: Wire proc‑macro**

```rust
#[proc_macro]
pub fn effect(input: TokenStream) -> TokenStream {
    let effect = parse_macro_input!(input as effect::Effect);
    // feature dispatch to appropriate emitter
}
```

- [ ] **Step 6: Add trybuild test**

```rust
// tests/ui/effect_pass.rs
use quoin_macros::effect;
fn test() {
    let count = 0;
    effect! { watch: [count], || println!("{}", count.get()) }
}
```

- [ ] **Step 7: Commit**

```bash
git add quoin-macros/src/effect.rs quoin-macros/src/lib.rs quoin-macros/tests/ui/effect_pass.rs
git commit -m "feat(quoin-macros): add effect! macro for reactive side effects"
```

### Task 2: `use_global` Hook

**Files:**
- Modify: `quoin/src/reactive.rs`
- Modify: `quoin-gpui/src/lib.rs`
- Modify: `quoin-leptos/src/lib.rs`
- Modify: `quoin-dioxus/src/lib.rs`
- Modify: `quoin-conformance/src/lib.rs`

- [ ] **Step 1: Extend `ReactiveContext` trait**

```rust
pub trait ReactiveContext: Clone + Send + Sync + 'static {
    // ... existing ...
    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>>;
}
```

- [ ] **Step 2: Implement for GPUI (stub—global state not yet integrated)**

```rust
impl ReactiveContext for GpuiContext {
    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        None // Placeholder; full impl requires global store
    }
}
```

- [ ] **Step 3: Implement for Leptos using `use_context`**

```rust
impl ReactiveContext for LeptosContext {
    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        use leptos::prelude::use_context;
        use_context::<leptos::prelude::RwSignal<T>>().map(|sig| LeptosSignal { inner: sig })
    }
}
```

- [ ] **Step 4: Implement for Dioxus using `use_context`**

```rust
impl ReactiveContext for DioxusContext {
    fn use_global<T: Clone + 'static + Send + Sync>(&self) -> Option<Self::Signal<T>> {
        dioxus::prelude::use_context::<Signal<T>>().map(|sig| DioxusSignal { inner: RefCell::new(sig) })
    }
}
```

- [ ] **Step 5: Add conformance test (optional, as GPUI returns `None`)**

- [ ] **Step 6: Commit**

```bash
git add quoin/src/reactive.rs quoin-gpui/src/lib.rs quoin-leptos/src/lib.rs quoin-dioxus/src/lib.rs quoin-conformance/src/lib.rs
git commit -m "feat(quoin): add use_global hook to ReactiveContext"
```

---

## Chunk 2: Button & Input UCP Adapters

### Task 3: GPUI Button

**Files:**
- Modify: `quoin-ui-gpui/src/lib.rs`
- Modify: `quoin-macros/src/emit/render_gpui.rs`

- [ ] **Step 1: Define `ButtonVariant` and `render_button`**

```rust
pub struct ButtonVariant {
    pub primary: bool,
    pub ghost: bool,
    pub size: ComponentSize,
}

pub fn render_button(
    label: Option<String>,
    icon: Option<IconName>,
    variant: ButtonVariant,
    on_click: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
) -> gpui::AnyElement {
    let mut btn = gpui_component::button::Button::new("quoin-btn");
    if let Some(l) = label { btn = btn.label(l); }
    if let Some(i) = icon { btn = btn.icon(i); }
    if variant.primary { btn = btn.primary(); }
    if variant.ghost { btn = btn.ghost(); }
    btn = btn.with_size(variant.size);
    if let Some(h) = on_click { btn = btn.on_click(move |_, w, cx| h(w, cx)); }
    btn.into_any_element()
}
```

- [ ] **Step 2: Add `emit_button` in `render_gpui.rs`**

```rust
fn emit_button(el: &Element) -> TokenStream {
    let label = find_arg_string(el, "label");
    let icon = find_arg_string(el, "icon");
    let primary = find_arg_bool(el, "primary");
    let ghost = find_arg_bool(el, "ghost");
    let on_click = find_arg_expr(el, "on_click");
    quote! {
        quoin_ui_gpui::render_button(
            #label,
            #icon,
            quoin_ui_gpui::ButtonVariant { primary: #primary, ghost: #ghost, size: quoin_ui_gpui::ComponentSize::Medium },
            #on_click
        )
    }
}
```

- [ ] **Step 3: Route `button` element to `emit_button`**

- [ ] **Step 4: Add trybuild test**

- [ ] **Step 5: Commit**

### Task 4: Leptos & Dioxus Button

- [ ] **Step 1: Leptos `render_button` using `leptos_shadcn_ui::Button`**

- [ ] **Step 2: Dioxus `render_button` using `shadcn_dioxus::Button`**

- [ ] **Step 3: Update emitters accordingly**

- [ ] **Step 4: Commit**

### Task 5: TextInput UCP (All Frameworks)

- [ ] **Step 1: GPUI `render_input` using `gpui_component::input::Input`**

- [ ] **Step 2: Leptos/Dioxus implementations**

- [ ] **Step 3: Emitters updated**

- [ ] **Step 4: Commit**

---

## Chunk 3: Complex UCP Components

### Task 6: DataTable (GPUI)

**Files:**
- Modify: `quoin-ui-gpui/src/lib.rs`
- Modify: `quoin-macros/src/emit/render_gpui.rs`

- [ ] **Step 1: Implement `render_table` with `DataTable` and delegate generation**

- [ ] **Step 2: Update `emit_data_table` to call it**

- [ ] **Step 3: Add test with columns, sorting**

- [ ] **Step 4: Commit**

### Task 7: DataTable (Leptos/Dioxus)

- [ ] **Step 1: Leptos using `leptos-shadcn-ui` Table**

- [ ] **Step 2: Dioxus using `shadcn-dioxus` Table**

- [ ] **Step 3: Emitters updated**

- [ ] **Step 4: Commit**

### Task 8: VirtualList (All Frameworks)

- [ ] **Step 1: GPUI `render_virtual_list` with scroll handle**

- [ ] **Step 2: Leptos/Dioxus implementations**

- [ ] **Step 3: Emitters updated**

- [ ] **Step 4: Commit**

### Task 9: DropdownMenu (All Frameworks)

- [ ] **Step 1: GPUI `render_dropdown` with `PopupMenu`**

- [ ] **Step 2: Leptos/Dioxus shadcn dropdown**

- [ ] **Step 3: Emitters updated**

- [ ] **Step 4: Commit**

### Task 10: TabBar (All Frameworks)

- [ ] **Step 1: GPUI `render_tabs` with `TabBar`**

- [ ] **Step 2: Leptos/Dioxus shadcn tabs**

- [ ] **Step 3: Emitters updated**

- [ ] **Step 4: Commit**

---

## Chunk 4: Styling Enhancements

### Task 11: Expand Tailwind Mapping

**Files:**
- Modify: `quoin-macros/src/transpile/tailwind.rs`

- [ ] **Step 1: Add opacity utilities (`opacity-*`)**

- [ ] **Step 2: Add `hover:` pseudo‑class parsing**

- [ ] **Step 3: Map hover variants to GPUI `.hover()` builder**

- [ ] **Step 4: Test with hover styles**

- [ ] **Step 5: Commit**

### Task 12: Theming via `ThemeToken`

- [ ] **Step 1: Resolve theme colors from `cx.theme()` in GPUI**

- [ ] **Step 2: Map to CSS variables in Leptos/Dioxus**

- [ ] **Step 3: Use in UCP components**

- [ ] **Step 4: Commit**

---

## Chunk 5: Clipboard Service

### Task 13: Clipboard Trait and Implementations

**Files:**
- Create: `quoin-ui/src/clipboard.rs`
- Modify: `quoin-ui-gpui/src/lib.rs`
- Modify: `quoin-ui-leptos/src/lib.rs`
- Modify: `quoin-ui-dioxus/src/lib.rs`

- [ ] **Step 1: Define `Clipboard` trait**

```rust
pub trait Clipboard {
    fn write_text(&self, text: &str);
}
```

- [ ] **Step 2: GPUI implementation using `cx.write_to_clipboard`**

- [ ] **Step 3: Leptos/Dioxus using `web_sys::Clipboard`**

- [ ] **Step 4: Add `use_clipboard` to `ReactiveContext`**

- [ ] **Step 5: Commit**

---

## Chunk 6: Validation – Mini Devtools Demo

### Task 14: Build Simplified Devtools Panel in `quoin`

**Files:**
- Create: `examples/mini-devtools/src/main.rs` (GPUI)
- Create: `examples/mini-devtools-leptos/src/lib.rs`

- [ ] **Step 1: Define state with `component!` (count, timeline events, cache entries)**

- [ ] **Step 2: Implement tabs using `TabBar` UCP**

- [ ] **Step 3: Implement timeline using `VirtualList`**

- [ ] **Step 4: Implement cache table using `DataTable`**

- [ ] **Step 5: Add filter dropdown using `DropdownMenu`**

- [ ] **Step 6: Verify all interactions work in GPUI**

- [ ] **Step 7: Port to Leptos and verify**

- [ ] **Step 8: Commit**

---

**Plan complete and saved to `docs/superpowers/plans/2026-04-20-quoin-ucp-devtools-gap-closure.md`. Ready to execute?**
