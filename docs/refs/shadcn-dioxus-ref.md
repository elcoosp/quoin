Below is the full API reference for the **shadcn‑dioxus** component library (Dioxus 0.7), derived directly from the provided source files. Each section lists the public components, their props, context structures (when relevant), sub‑components, and usage notes. All types are assumed to be in the `ui` crate unless noted otherwise.

---

# shadcn-dioxus – Full Component API Reference

## Important Global Notes

- All components live under the `ui` crate (`packages/ui`).
- Many overlay components (dialog, sheet, popover, tooltip, etc.) require your app to be wrapped with `PortalProvider` (from `ui::PortalProvider`) for proper rendering.
- Some components expose a **context** that is consumed by child sub‑components using `use_context`. The context is automatically provided by the parent component – no manual setup is needed.
- All component props are **owned** (`String` instead of `&str`, `Vec<T>` instead of `&[T]`). They implement `Clone + PartialEq`.
- Signals and callbacks are used extensively for state management.
- For a default Tailwind CSS theme, include the stylesheet from `packages/web/tailwind.css` or the generated `assets/tailwind.css`.

---

## Accordion

An accordion is a vertically stacked set of interactive headings that reveal or hide content panels.

**Context:** `AccordionContext` – shared state (`open_items`, `set_open_items`, `multiple`).

**Main components:**

- `Accordion`
- `AccordionItem`
- `AccordionTrigger`
- `AccordionContent`

### Accordion

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AccordionProps {
    #[props(default = false)]
    pub multiple: bool,
    #[props(default)]
    pub default_value: Vec<String>,
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

- `multiple` – if `true`, multiple items can be open simultaneously; otherwise only one.
- `default_value` – initial list of `value` strings for open items.

### AccordionItem

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AccordionItemProps {
    #[props(into)]
    pub value: String,
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

- `value` – unique identifier for this item.

### AccordionTrigger

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AccordionTriggerProps {
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

### AccordionContent

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AccordionContentProps {
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

Content is only rendered when its item is open.

**Usage example:**

```rust
rsx! {
    Accordion { multiple: false, default_value: vec!["item-1".into()],
        AccordionItem { value: "item-1".into(),
            AccordionTrigger { "Trigger 1" }
            AccordionContent { "Content 1" }
        }
        AccordionItem { value: "item-2".into(),
            AccordionTrigger { "Trigger 2" }
            AccordionContent { "Content 2" }
        }
    }
}
```

---

## Alert

Displays a callout with title and description.

**Variants:** `AlertVariant` – `Default`, `Destructive`.

### Alert

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AlertProps {
    #[props(default)]
    pub variant: AlertVariant,
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

### AlertTitle

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AlertTitleProps {
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

### AlertDescription

```rust
#[derive(Clone, PartialEq, Props)]
pub struct AlertDescriptionProps {
    #[props(into, default)]
    pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    pub attributes: Vec<Attribute>,
}
```

**Usage:**

```rust
rsx! {
    Alert { variant: AlertVariant::Destructive,
        AlertTitle { "Error" }
        AlertDescription { "Something went wrong." }
    }
}
```

---

## Alert Dialog

A modal dialog that requires user action. Uses the same portal pattern as `Dialog` (requires `PortalProvider`).

**Context:** `AlertDialogContext` – `open` signal and `set_open` callback.

**Components:**

- `AlertDialog`
- `AlertDialogTrigger`
- `AlertDialogContent`
- `AlertDialogHeader`
- `AlertDialogFooter`
- `AlertDialogTitle`
- `AlertDialogDescription`
- `AlertDialogAction`
- `AlertDialogCancel`

### AlertDialog

```rust
pub struct AlertDialogProps { pub children: Element }
```

### AlertDialogTrigger

```rust
pub struct AlertDialogTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### AlertDialogContent

```rust
pub struct AlertDialogContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = true)] pub show_close_button: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

- `show_close_button` – adds an X button to close the dialog.

### AlertDialogHeader

```rust
pub struct AlertDialogHeaderProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### AlertDialogFooter

```rust
pub struct AlertDialogFooterProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### AlertDialogTitle

```rust
pub struct AlertDialogTitleProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### AlertDialogDescription

```rust
pub struct AlertDialogDescriptionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### AlertDialogAction

```rust
pub struct AlertDialogActionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### AlertDialogCancel

```rust
pub struct AlertDialogCancelProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Usage:**

```rust
rsx! {
    AlertDialog {
        AlertDialogTrigger { "Show dialog" }
        AlertDialogContent {
            AlertDialogHeader {
                AlertDialogTitle { "Delete item" }
                AlertDialogDescription { "This action cannot be undone." }
            }
            AlertDialogFooter {
                AlertDialogCancel { "Cancel" }
                AlertDialogAction { "Delete" }
            }
        }
    }
}
```

---

## Aspect Ratio

Keeps content in a specific aspect ratio.

```rust
#[derive(Props, Clone, PartialEq)]
pub struct AspectRatioProps {
    #[props(default = 1.0)] pub ratio: f64,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

- `ratio` – width / height (e.g. 16/9 = 1.777...).

```rust
rsx! {
    AspectRatio { ratio: 1.5, img { ... } }
}
```

---

## Avatar

An image with a fallback (initials / placeholder).

**Context:** `AvatarCtx` – loading state (`Signal<AvatarLoadingStatus>`). The loading status is `Idle`, `Loading`, `Loaded`, or `Error`.

**Components:**

- `Avatar`
- `AvatarImage`
- `AvatarFallback`

### Avatar

```rust
pub struct AvatarProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = span)] pub attributes: Vec<Attribute>,
}
```

### AvatarImage

```rust
pub struct AvatarImageProps {
    pub src: String,
    #[props(default)] pub alt: String,
    #[props(extends = img)] pub attributes: Vec<Attribute>,
}
```

The image will fire `onload` and `onerror` to update the context state.

### AvatarFallback

```rust
pub struct AvatarFallbackProps {
    #[props(default)] pub delay_ms: Option<u64>,
    pub children: Element,
    #[props(extends = span)] pub attributes: Vec<Attribute>,
}
```

- `delay_ms` – wait before showing the fallback (in case the image is still loading).

**Usage:**

```rust
rsx! {
    Avatar {
        AvatarImage { src: "url", alt: "User" }
        AvatarFallback { "AB" }
    }
}
```

---

## Badge

Small status descriptors.

**Variants:** `BadgeVariant` – `Default`, `Secondary`, `Destructive`, `Outline`.

```rust
pub struct BadgeProps {
    #[props(default)] pub variant: BadgeVariant,
    #[props(into, default)] pub class: String,
    pub href: Option<String>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

- `href` – if provided, renders an anchor tag instead of a `<span>`.

Helper: `badge_variants(variant)` returns combined Tailwind classes.

```rust
rsx! {
    Badge { variant: BadgeVariant::Outline, "New" }
    Badge { href: "/docs", "Link Badge" }
}
```

---

## Breadcrumb

Hierarchical navigation path.

**Components:** `Breadcrumb`, `BreadcrumbList`, `BreadcrumbItem`, `BreadcrumbLink`, `BreadcrumbPage`, `BreadcrumbSeparator`.

### Breadcrumb

`Breadcrumb` is just a `<nav>` wrapper.

```rust
pub struct BreadcrumbProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### BreadcrumbList

```rust
pub struct BreadcrumbListProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### BreadcrumbItem

```rust
pub struct BreadcrumbItemProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### BreadcrumbLink

```rust
pub struct BreadcrumbLinkProps {
    #[props(into, default)] pub class: String,
    #[props(default)] pub href: Option<String>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### BreadcrumbPage

```rust
pub struct BreadcrumbPageProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### BreadcrumbSeparator

```rust
pub struct BreadcrumbSeparatorProps {
    #[props(into, default)] pub class: String,
}
```

**Usage:**

```rust
rsx! {
    Breadcrumb {
        BreadcrumbList {
            BreadcrumbItem { BreadcrumbLink { href: "/", "Home" } }
            BreadcrumbSeparator {}
            BreadcrumbItem { BreadcrumbPage { "Current" } }
        }
    }
}
```

---

## Button

**Variants:** `ButtonVariant` – `Default`, `Destructive`, `Outline`, `Secondary`, `Ghost`, `Link`.  
**Sizes:** `ButtonSize` – `Default`, `Sm`, `Lg`, `Icon`, `IconSm`, `IconLg`.

```rust
pub struct ButtonProps {
    #[props(default)] pub variant: ButtonVariant,
    #[props(default)] pub size: ButtonSize,
    #[props(into, default)] pub class: String,
    pub href: Option<String>,
    #[props(default = "button".to_string())] pub button_type: String,
    #[props(default = false)] pub disabled: bool,
    pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

- `href` – if set, renders as `<a>`; otherwise `<button>`.
- `button_type` – e.g. `"submit"`, `"reset"`.

Also exposed: `button_variants(variant, size) -> String` to get Tailwind classes for use on e.g. `<a>` tags.

```rust
rsx! {
    Button { variant: ButtonVariant::Outline, size: ButtonSize::Sm, "Click" }
    a { class: button_variants(ButtonVariant::Ghost, ButtonSize::Default), href: "/", "Link" }
}
```

---

## Button Group

Groups related buttons with optional separators.

**Re‑exports under `ButtonGroup` module:**
- `ButtonGroup::Root` (the `ButtonGroup` component)
- `ButtonGroup::Text`
- `ButtonGroup::Separator`

### ButtonGroup (Root)

- `orientation: ButtonGroupOrientation` (Horizontal / Vertical)
- `class: String`
- `children: Element`

```rust
rsx! {
    ButtonGroup::Root { orientation: ButtonGroupOrientation::Vertical,
        Button { "A" }
        ButtonGroup::Separator {}
        Button { "B" }
    }
}
```

### ButtonGroupText

Renders text inside a group.

```rust
pub struct ButtonGroupTextProps {
    class: Option<String>,
    children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ButtonGroupSeparator

Visual divider.

```rust
pub struct ButtonGroupSeparatorProps {
    #[props(default)] pub orientation: SeparatorOrientation,
    #[props(into, default)] pub class: String,
    #[props(default = true)] pub decorative: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

---

## Calendar

A monthly calendar grid with date selection.

**Context:** `CalendarContext` – `year`, `month`, `selected_date` and `set_selected_date`.

**Sub‑components:** `CalendarHeader`, `CalendarGrid`, `CalendarDay` (internal).

### Calendar

```rust
pub struct CalendarProps {
    #[props(default = 2024)] pub default_year: i32,
    #[props(default = 0)] pub default_month: u32,
    #[props(default)] pub default_date: Option<(i32, u32, u32)>, // (year, month, day)
    #[props(default)] pub on_change: Option<Callback<Option<(i32, u32, u32)>>>,
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

`default_date` can pre‑select a date. `on_change` fires with `Some((y,m,d))` or `None`.

```rust
rsx! {
    Calendar {
        default_year: 2025,
        default_month: 5,
        on_change: move |date| { /* ... */ },
    }
}
```

---

## Card

A container with header, content, and footer slots.

**Components:** `Card`, `CardHeader`, `CardTitle`, `CardDescription`, `CardContent`, `CardFooter`, `CardAction`.

### Card

```rust
pub struct CardProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CardHeader

```rust
pub struct CardHeaderProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CardTitle

```rust
pub struct CardTitleProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CardDescription

```rust
pub struct CardDescriptionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CardContent

```rust
pub struct CardContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CardFooter

```rust
pub struct CardFooterProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CardAction

Used inside `CardHeader` to place an action (like a button) on the right.

```rust
pub struct CardActionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Card {
        CardHeader {
            CardTitle { "Title" }
            CardAction { button { "Edit" } }
        }
        CardContent { "Content" }
        CardFooter { "Footer" }
    }
}
```

---

## Carousel

Slides through content items horizontally or vertically.

**Context:** `CarouselContext` – `current_index`, `total`, `set_index`, `orientation`, `auto_play`.

**Components:** `Carousel`, `CarouselContent`, `CarouselItem`, `CarouselPrevious`, `CarouselNext`, `CarouselIndicators`.

### Carousel

```rust
pub struct CarouselProps {
    #[props(default)] pub orientation: CarouselOrientation, // Horizontal or Vertical
    #[props(default = false)] pub auto_play: bool,
    #[props(default = 3000)] pub auto_play_interval_ms: u64,
    pub total: usize, // number of items
    #[props(default = "20rem".to_string())] pub height: String,
    #[props(default = "100%".to_string())] pub width: String,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CarouselContent

Wrapper that applies the transform.

```rust
pub struct CarouselContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CarouselItem

```rust
pub struct CarouselItemProps {
    pub index: usize, // not strictly required but used as key
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CarouselPrevious / CarouselNext

```rust
pub struct CarouselPreviousProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
// Same for CarouselNextProps.
```

### CarouselIndicators

```rust
pub struct CarouselIndicatorsProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Carousel { total: 3, width: "100%", height: "400px",
        CarouselContent {
            CarouselItem { index: 0, "Slide 1" }
            CarouselItem { index: 1, "Slide 2" }
            CarouselItem { index: 2, "Slide 3" }
        }
        CarouselPrevious {}
        CarouselNext {}
        CarouselIndicators {}
    }
}
```

---

## Chart

Simple bar chart using SVG.

```rust
pub struct ChartBar {
    pub label: String,
    pub value: f64,
    pub color: String,
}

pub struct ChartProps {
    #[props(default = 200.0)] pub height: f64,
    #[props(default = 400.0)] pub width: f64,
    #[props(default = 4.0)] pub bar_gap: f64,
    pub bars: Vec<ChartBar>,
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

```rust
rsx! {
    Chart { bars: vec![ChartBar{label:"A".into(), value:30.0, color:"red".into()}] }
}
```

---

## Checkbox

Three‑state checkbox.

**State:** `CheckboxState` – `Checked`, `Unchecked`, `Indeterminate`.

```rust
pub struct CheckboxProps {
    #[props(default)] pub checked: Option<Signal<CheckboxState>>,
    #[props(default)] pub default_checked: CheckboxState,
    #[props(default = false)] pub disabled: bool,
    #[props(into, default)] pub class: String,
    #[props(default)] pub on_checked_change: Option<Callback<CheckboxState>>,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

- Controlled: pass `checked` signal; uncontrolled: use `default_checked`.

```rust
rsx! {
    Checkbox {
        checked: my_signal,
        on_checked_change: move |state| { /* ... */ },
    }
}
```

---

## Collapsible

A container that expands/collapses its content.

**Context:** `CollapsibleContext` – `open` and `set_open`.

**Components:** `Collapsible`, `CollapsibleTrigger`, `CollapsibleContent`.

### Collapsible

```rust
pub struct CollapsibleProps {
    #[props(default = false)] pub default_open: bool,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CollapsibleTrigger

```rust
pub struct CollapsibleTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CollapsibleContent

```rust
pub struct CollapsibleContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Usage:**

```rust
rsx! {
    Collapsible {
        CollapsibleTrigger { "Toggle" }
        CollapsibleContent { "Hidden until open." }
    }
}
```

---

## Color Picker

Simple HSL slider‑based color picker.

```rust
pub struct ColorPickerProps {
    #[props(default = "0.0".to_string())] pub default_hue: String, // parsed to f64
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

```rust
rsx! {
    ColorPicker { default_hue: "200.0" }
}
```

---

## Combobox

Text input with dropdown suggestions.

```rust
pub struct ComboboxOption {
    pub value: String,
    pub label: String,
}

pub struct ComboboxProps {
    pub options: Vec<ComboboxOption>,
    #[props(default = "Select an option...".to_string())] pub placeholder: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

Filtering is case‑insensitive. No controlled/uncontrolled; state is internal.

```rust
rsx! {
    Combobox { options: vec![...] }
}
```

---

## Command

Command palette / searchable list.

**Context:** `CommandContext` – `value` (search), `set_value`, `selected_id`, `set_selected_id`, `visible_count`.

**Components:** `Command`, `CommandInput`, `CommandList`, `CommandEmpty`, `CommandGroup`, `CommandItem`, `CommandSeparator`, `CommandShortcut`.

### Command

```rust
pub struct CommandProps {
    #[props(default)] pub default_value: String,
    #[props(default)] pub on_select: Option<Callback<String>>,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandInput

```rust
pub struct CommandInputProps {
    #[props(default = "Type a command or search...".to_string())] pub placeholder: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandList

```rust
pub struct CommandListProps {
    #[props(into, default)] pub class: String,
    #[props(default = 200)] pub max_height: u32,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandGroup

```rust
pub struct CommandGroupProps {
    #[props(into, default)] pub heading: Option<String>,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandItem

```rust
pub struct CommandItemProps {
    #[props(into)] pub value: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub on_select: Option<Callback<()>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandEmpty

Shown when no items match.

```rust
pub struct CommandEmptyProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandSeparator

```rust
pub struct CommandSeparatorProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### CommandShortcut

Renders keyboard shortcut hint.

```rust
pub struct CommandShortcutProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Command {
        CommandInput { placeholder: "Search..." }
        CommandList {
            CommandGroup { heading: Some("Actions".into()),
                CommandItem { value: "open", "Open" }
                CommandItem { value: "close", "Close" }
            }
        }
        CommandEmpty { "No results." }
    }
}
```

---

## Context Menu

Right‑click contextual menu.

**Context:** `ContextMenuContext` – `open`, `set_open`.

**Components:** `ContextMenu`, `ContextMenuTrigger`, `ContextMenuContent`, `ContextMenuItem`, `ContextMenuSeparator`.

### ContextMenu

```rust
pub struct ContextMenuProps { pub children: Element }
```

### ContextMenuTrigger

The element (div) that listens for contextmenu event.

```rust
pub struct ContextMenuTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ContextMenuContent

```rust
pub struct ContextMenuContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub force_mount: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ContextMenuItem

```rust
pub struct ContextMenuItemProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ContextMenuSeparator

```rust
pub struct ContextMenuSeparatorProps {
    #[props(into, default)] pub class: String,
}
```

**Example:**

```rust
rsx! {
    ContextMenu {
        ContextMenuTrigger { "Right-click me" }
        ContextMenuContent {
            ContextMenuItem { "Copy" }
            ContextMenuSeparator {}
            ContextMenuItem { "Delete" }
        }
    }
}
```

---

## Data Table

Feature‑rich table with sorting, filtering, pagination, column visibility.

**Context:** `DataTableContext` – many signals; automatically provided by `DataTable`.

**Components:** `DataTable`, `DataTableToolbar`, `DataTableHeader`, `DataTableBody`, `DataTablePagination`.

### DataTable

```rust
pub struct DataTableProps {
    pub columns: Vec<DataTableColumn>,
    pub rows: Vec<Vec<String>>,  // each inner Vec contains the cell values
    #[props(default = 10)] pub page_size: usize,
    #[props(default = true)] pub show_toolbar: bool,
    #[props(default = true)] pub show_pagination: bool,
    #[props(default)] pub on_selection_change: Option<Callback<Vec<usize>>>,
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

`DataTableColumn`:
```rust
pub struct DataTableColumn {
    pub id: String,
    pub header: String,
    pub sortable: bool,
}
```

### DataTableToolbar

Contains filter input and column visibility dropdown.

```rust
pub struct DataTableToolbarProps {
    #[props(default = true)] pub show_column_visibility: bool,
    #[props(into, default)] pub class: String,
}
```

### DataTableHeader

(Self‑contained, no props exposed.)

### DataTableBody

(Self‑contained, no props exposed.)

### DataTablePagination

```rust
pub struct DataTablePaginationProps {
    #[props(default = vec![10, 20, 30, 40, 50])] pub page_size_options: Vec<usize>,
    #[props(into, default)] pub class: String,
}
```

**Example:**

```rust
rsx! {
    DataTable {
        columns: vec![DataTableColumn::new("name", "Name")],
        rows: vec![vec!["Alice".into()], vec!["Bob".into()]],
        page_size: 10,
    }
}
```

---

## Date Picker

Dropdown calendar date picker.

**Context:** `DatePickerContext` – `selected_date` and `set_selected_date`.

**Components:** `DatePicker`, `DatePickerTrigger`, `DatePickerContent`.

### DatePicker

```rust
pub struct DatePickerProps {
    #[props(default)] pub default_date: Option<(i32, u32, u32)>,
    #[props(default = "Pick a date".to_string())] pub placeholder: String,
    #[props(into, default)] pub class: String,
    #[props(default)] pub on_change: Option<Callback<Option<(i32, u32, u32)>>>,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DatePickerTrigger

```rust
pub struct DatePickerTriggerProps {
    #[props(default = "Pick a date".to_string())] pub placeholder: String,
    #[props(default = false)] pub disabled: bool,
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DatePickerContent

(Internal – uses Calendar and Popover context.)

**Usage:**

```rust
rsx! {
    DatePicker { on_change: move |date| { /* ... */ }, placeholder: "Select date" }
}
```

---

## Dialog

Modal overlay dialog. **Requires `PortalProvider`**.

**Context:** `DialogContext` – `open`, `set_open`, accessibility IDs.

**Components:**
- `Dialog`
- `DialogTrigger`
- `DialogPortal`
- `DialogOverlay`
- `DialogContent`
- `DialogHeader`
- `DialogTitle`
- `DialogDescription`
- `DialogFooter`
- `DialogClose`

### Dialog

```rust
pub struct DialogProps {
    #[props(default)] pub open: Option<Signal<bool>>,
    #[props(default = false)] pub default_open: bool,
    #[props(default)] pub on_open_change: Option<Callback<bool>>,
    pub children: Element,
}
```

- Controlled: provide `open` signal; uncontrolled: `default_open`.

### DialogTrigger

```rust
pub struct DialogTriggerProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogPortal

Portals the overlay and content to the app root.

```rust
pub struct DialogPortalProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogOverlay

Click to close backdrop.

```rust
pub struct DialogOverlayProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogContent

```rust
pub struct DialogContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = true)] pub show_close_button: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogHeader

```rust
pub struct DialogHeaderProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogTitle

```rust
pub struct DialogTitleProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogDescription

```rust
pub struct DialogDescriptionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogFooter

```rust
pub struct DialogFooterProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DialogClose

Wraps a button to close the dialog.

```rust
pub struct DialogCloseProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Usage:**

```rust
rsx! {
    Dialog {
        DialogTrigger { "Open" }
        DialogPortal {
            DialogOverlay {}
            DialogContent {
                DialogHeader {
                    DialogTitle { "Title" }
                    DialogDescription { "Description" }
                }
                DialogFooter {
                    DialogClose { "Cancel" }
                    Button { "Save" }
                }
            }
        }
    }
}
```

> **Note:** If nesting dialog portals, wrap each portal’s content with `DialogContextProvider` (internal, but exposed for advanced use).

---

## Drawer

Side panel that slides in from an edge.

**Context:** `DrawerContext` – `open`, `set_open`.

**Components:** `Drawer`, `DrawerTrigger`, `DrawerContent`, `DrawerHeader`, `DrawerFooter`, `DrawerTitle`, `DrawerDescription`, `DrawerClose`, `DrawerOverlay`.

### Drawer

```rust
pub struct DrawerProps { pub children: Element }
```

### DrawerTrigger

```rust
pub struct DrawerTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DrawerContent

`DrawerSide` enum: `Right`, `Left`, `Top`, `Bottom`.

```rust
pub struct DrawerContentProps {
    #[props(default)] pub side: DrawerSide,
    #[props(into, default)] pub class: String,
    #[props(default = true)] pub show_close_button: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DrawerOverlay

```rust
pub struct DrawerOverlayProps {
    #[props(into, default)] pub class: String,
}
```

### DrawerHeader, DrawerFooter, DrawerTitle, DrawerDescription, DrawerClose

These have simple `class` and `children` props.

**Example:**

```rust
rsx! {
    Drawer {
        DrawerTrigger { "Open" }
        DrawerOverlay {}
        DrawerContent { side: DrawerSide::Right,
            DrawerHeader { DrawerTitle { "Settings" } }
            DrawerFooter { DrawerClose { "Close" } }
        }
    }
}
```

---

## Dropdown Menu

Button‑triggered overlay menu.

**Context:** `DropdownMenuContext` – `open`, `set_open`.

**Components:** `DropdownMenu`, `DropdownMenuTrigger`, `DropdownMenuContent`, `DropdownMenuItem`, `DropdownMenuLabel`, `DropdownMenuSeparator`, `DropdownMenuShortcut`.

### DropdownMenu

```rust
pub struct DropdownMenuProps { pub children: Element }
```

### DropdownMenuTrigger

```rust
pub struct DropdownMenuTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DropdownMenuContent

```rust
pub struct DropdownMenuContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = "bottom".to_string())] pub side: String, // "top", "bottom", "left", "right"
    #[props(default = false)] pub force_mount: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DropdownMenuItem

```rust
pub struct DropdownMenuItemProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### DropdownMenuLabel

```rust
pub struct DropdownMenuLabelProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### DropdownMenuSeparator

```rust
pub struct DropdownMenuSeparatorProps {
    #[props(into, default)] pub class: String,
}
```

### DropdownMenuShortcut

```rust
pub struct DropdownMenuShortcutProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

**Example:**

```rust
rsx! {
    DropdownMenu {
        DropdownMenuTrigger { Button { "Menu" } }
        DropdownMenuContent { side: "bottom",
            DropdownMenuLabel { "Actions" }
            DropdownMenuItem { onclick: move |_| {}, "Edit" }
            DropdownMenuSeparator {}
            DropdownMenuItem { "Delete" }
        }
    }
}
```

---

## Empty

Empty state container with icon, title, description, and action area.

**Components:** `Empty`, `EmptyHeader`, `EmptyMedia`, `EmptyTitle`, `EmptyDescription`, `EmptyContent`.

### Empty

```rust
pub struct EmptyProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### EmptyHeader

```rust
pub struct EmptyHeaderProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### EmptyMedia

Variants: `Default`, `Icon`.

```rust
pub struct EmptyMediaProps {
    #[props(default)] pub variant: EmptyMediaVariant,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### EmptyTitle

```rust
pub struct EmptyTitleProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### EmptyDescription

```rust
pub struct EmptyDescriptionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### EmptyContent

```rust
pub struct EmptyContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Empty {
        EmptyHeader {
            EmptyMedia { variant: EmptyMediaVariant::Icon, Icon {...} }
            EmptyTitle { "No items" }
            EmptyDescription { "Add your first item." }
        }
        EmptyContent {
            Button { "Add Item" }
        }
    }
}
```

---

## Field

A robust form field layout system supporting labels, descriptions, errors, and various orientations.

**Components:** `Field`, `FieldLabel`, `FieldDescription`, `FieldError`, `FieldContent`, `FieldGroup`, `FieldSet`, `FieldLegend`, `FieldSeparator`, `FieldTitle`.

### Field

Orientation: `Vertical`, `Horizontal`, `Responsive`.

```rust
pub struct FieldProps {
    #[props(default)] pub orientation: FieldOrientation,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldLabel

Wraps the `Label` component with extra classes.

```rust
pub struct FieldLabelProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldDescription

```rust
pub struct FieldDescriptionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldError

`FieldErrorMessage` has an optional `message: Option<String>`. Multiple error messages render as a list.

```rust
pub struct FieldErrorProps {
    #[props(into, default)] pub class: String,
    #[props(default)] pub children: Element,
    #[props(default)] pub errors: Vec<FieldErrorMessage>,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldContent

Flex container for label + description.

```rust
pub struct FieldContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldGroup

Groups multiple fields vertically.

```rust
pub struct FieldGroupProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldSet

Semantic `<fieldset>`.

```rust
pub struct FieldSetProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldLegend

Variant: `Label` or `Legend`.

```rust
pub struct FieldLegendProps {
    #[props(default)] pub variant: FieldLegendVariant,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldSeparator

```rust
pub struct FieldSeparatorProps {
    #[props(into, default)] pub class: String,
    #[props(default)] pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### FieldTitle

```rust
pub struct FieldTitleProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Basic example:**

```rust
rsx! {
    Field {
        FieldLabel { "Email" }
        Input { r#type: InputType::Email, placeholder: "email" }
        FieldDescription { "We'll never share your email." }
    }
}
```

With validation:

```rust
rsx! {
    Field {
        FieldLabel { "Username" }
        Input { "aria-invalid": "true" }
        FieldError {
            errors: vec![FieldErrorMessage { message: Some("Required".into()) }]
        }
    }
}
```

---

## Hover Card

Popover that appears on hover.

**Context:** `HoverCardContext` – `open`, `set_open`.

**Components:** `HoverCard`, `HoverCardTrigger`, `HoverCardContent`.

### HoverCard

```rust
pub struct HoverCardProps { pub children: Element }
```

### HoverCardTrigger

```rust
pub struct HoverCardTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### HoverCardContent

```rust
pub struct HoverCardContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = "top".to_string())] pub side: String, // "top", "bottom", "left", "right"
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    HoverCard {
        HoverCardTrigger { span { "@username" } }
        HoverCardContent { side: "top",
            div { "User info" }
        }
    }
}
```

---

## Input

Styled `<input>` element.

**InputType:** `Text`, `Password`, `Email`, `Number`, `Tel`, `Url`, `Search`, `Date`, `Time`, `DatetimeLocal`, `Month`, `Week`, `Color`, `Hidden`, `File`.

```rust
pub struct InputProps {
    #[props(default)] pub r#type: InputType,
    #[props(into, default)] pub class: String,
    #[props(into, default = "input".to_string())] pub data_slot: String,
    // All standard event handlers...
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    pub attributes: Vec<Attribute>,
}
```

Helper: `input_classes(input_type) -> String`.

```rust
rsx! {
    Input { r#type: InputType::Email, placeholder: "Email" }
}
```

---

## Input Group

Composite input wrapper with add‑ons, buttons, labels.

**Context:** `InputGroupCtx` – holds a reference to the main control (input/textarea) for focusing.

**Components (re‑exported directly):** `InputGroup`, `InputGroupAddon`, `InputGroupButton`, `InputGroupInput`, `InputGroupText`, `InputGroupTextarea`.

### InputGroup

```rust
pub struct InputGroupProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### InputGroupInput

Should be used inside `InputGroup` instead of a plain `Input`.

```rust
pub struct InputGroupInputProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)]
    #[props(extends = input)]
    pub attributes: Vec<Attribute>,
}
```

### InputGroupTextarea

Similar to `InputGroupInput` but for `<textarea>`.

### InputGroupAddon

Alignment: `InputGroupAddonAlign` – `InlineStart`, `InlineEnd`, `BlockStart`, `BlockEnd`.

```rust
pub struct InputGroupAddonProps {
    #[props(default)] pub align: InputGroupAddonAlign,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### InputGroupButton

```rust
pub struct InputGroupButtonProps {
    #[props(default)] pub size: InputGroupButtonSize, // Xs, Sm, IconXs, IconSm
    #[props(default = ButtonVariant::Ghost)] pub variant: ButtonVariant,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    pub children: Element,
}
```

### InputGroupText

```rust
pub struct InputGroupTextProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    InputGroup {
        InputGroupAddon { "@" }
        InputGroupInput { placeholder: "Username" }
        InputGroupAddon { align: InputGroupAddonAlign::InlineEnd,
            InputGroupButton { size: InputGroupButtonSize::Sm, "Search" }
        }
    }
}
```

---

## Input OTP

One‑time password input with individual slots.

**Context:** `InputOtpContext` – `value`, `max_length`, `set_value`.

**Components:** `InputOtp`, `InputOtpGroup`, `InputOtpSlot`, `InputOtpSeparator`.

### InputOtp

```rust
pub struct InputOtpProps {
    #[props(default = 6)] pub max_length: usize,
    #[props(default)] pub value: String,
    #[props(default)] pub on_change: Option<Callback<String>>,
    pub children: Element,
}
```

### InputOtpGroup

Group of slots (optional).

```rust
pub struct InputOtpGroupProps { pub children: Element }
```

### InputOtpSlot

```rust
pub struct InputOtpSlotProps {
    #[props(default = 0)] pub index: usize,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
}
```

### InputOtpSeparator

```rust
// no props
```

**Example:**

```rust
rsx! {
    InputOtp { max_length: 6, on_change: move |v| {},
        InputOtpGroup {
            InputOtpSlot { index: 0 }
            InputOtpSlot { index: 1 }
            InputOtpSeparator {}
            InputOtpSlot { index: 2 }
        }
    }
}
```

---

## Item

Flexible card‑like container with variants, sizes, and media slots.

**Exports:**

- `Item` (root)
- `ItemContent`, `ItemTitle`, `ItemDescription`, `ItemActions`, `ItemMedia`, `ItemGroup`, `ItemSeparator`

### Item

Variants: `Default`, `Outline`, `Muted`. Sizes: `Default`, `Sm`.

```rust
pub struct ItemProps {
    #[props(default)] pub variant: ItemVariant,
    #[props(default)] pub size: ItemSize,
    #[props(into, default)] pub class: String,
    pub as_child: Option<RenderFn>, // for rendering as a link etc.
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

The `RenderFn` allows you to customize the element (e.g., wrap as `<a>`).

### ItemContent

```rust
pub struct ItemContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ItemTitle, ItemDescription

Simple `class` + `children`.

### ItemMedia

Variants: `Default`, `Icon`, `Image`.

```rust
pub struct ItemMediaProps {
    #[props(default)] pub variant: ItemMediaVariant,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ItemGroup

```rust
pub fn ItemGroup(#[props(into, default)] class: String, children: Element) -> Element
```

### ItemSeparator

```rust
pub struct ItemSeparatorProps {
    class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Item { variant: ItemVariant::Outline,
        ItemMedia { variant: ItemMediaVariant::Icon, Icon {} }
        ItemContent {
            ItemTitle { "Item" }
            ItemDescription { "Description" }
        }
        ItemActions { Button { "Action" } }
    }
}
```

---

## Kbd

Keyboard key indicator.

**Components:** `Kbd`, `KbdGroup`.

### Kbd

```rust
pub struct KbdProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### KbdGroup

```rust
pub struct KbdGroupProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    KbdGroup {
        Kbd { "Ctrl" }
        Kbd { "C" }
    }
}
```

---

## Label

Accessible `<label>` element.

```rust
pub struct LabelProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

Use with `r#for` attribute to associate with inputs.

```rust
rsx! {
    Label { r#for: "email", "Email" }
    Input { id: "email", ... }
}
```

---

## Menubar

Desktop‑style horizontal menu bar.

**Context:** `MenubarContext` – tracks which menu is open.

**Components:** `Menubar`, `MenubarMenu`, `MenubarTrigger`, `MenubarContent`, `MenubarItem`, `MenubarSeparator`.

### Menubar

```rust
pub struct MenubarProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### MenubarMenu

Each menu inside the bar.

```rust
pub struct MenubarMenuProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### MenubarTrigger

```rust
pub struct MenubarTriggerProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### MenubarContent

```rust
pub struct MenubarContentProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### MenubarItem

```rust
pub struct MenubarItemProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
}
```

### MenubarSeparator

```rust
pub struct MenubarSeparatorProps {
    #[props(into, default)] pub class: String,
}
```

**Example:**

```rust
rsx! {
    Menubar {
        MenubarMenu {
            MenubarTrigger { "File" }
            MenubarContent {
                MenubarItem { onclick: move |_| {}, "New" }
                MenubarSeparator {}
                MenubarItem { "Exit" }
            }
        }
    }
}
```

---

## Native Select

Styled native `<select>`.

**Components:** `NativeSelect`, `NativeSelectOption`, `NativeSelectOptGroup`.

### NativeSelect

```rust
pub struct NativeSelectProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    #[props(extends = select)]
    pub attributes: Vec<Attribute>,
}
```

### NativeSelectOption

```rust
pub struct NativeSelectOptionProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    #[props(extends = option)]
    pub attributes: Vec<Attribute>,
}
```

### NativeSelectOptGroup

```rust
pub struct NativeSelectOptGroupProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)]
    #[props(extends = option)]
    pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    NativeSelect {
        NativeSelectOption { value: "1", "Option 1" }
    }
}
```

---

## Navigation Menu

Horizontal or vertical navigation with dropdown sub‑menus.

**Context:** `NavigationMenuContext` (global for the menu) and `NavigationMenuItemContext` (per item).

**Components:** `NavigationMenu`, `NavigationMenuList`, `NavigationMenuItem`, `NavigationMenuTrigger`, `NavigationMenuContent`, `NavigationMenuLink`, `NavigationMenuIndicator`.

Also: `navigation_menu_trigger_style()` helper.

### NavigationMenu

Orientation: `Horizontal` or `Vertical`.

```rust
pub struct NavigationMenuProps {
    #[props(default)] pub orientation: NavigationMenuOrientation,
    #[props(default = false)] pub skip_delay_show: bool,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### NavigationMenuList

```rust
pub struct NavigationMenuListProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### NavigationMenuItem

```rust
pub struct NavigationMenuItemProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### NavigationMenuTrigger

```rust
pub struct NavigationMenuTriggerProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### NavigationMenuContent

```rust
pub struct NavigationMenuContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub force_mount: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### NavigationMenuLink

```rust
pub struct NavigationMenuLinkProps {
    #[props(into, default)] pub class: String,
    #[props(default)] pub href: Option<String>,
    #[props(default = false)] pub active: bool,
    #[props(default)] pub onselect: Option<Callback<()>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### NavigationMenuIndicator

Small arrow below the trigger.

```rust
pub struct NavigationMenuIndicatorProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub force_mount: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    NavigationMenu {
        NavigationMenuList {
            NavigationMenuItem {
                NavigationMenuTrigger { "Products" }
                NavigationMenuContent {
                    NavigationMenuLink { href: "/item1", "Item 1" }
                }
            }
            NavigationMenuItem {
                NavigationMenuLink { href: "/about", "About" }
            }
        }
    }
}
```

---

## Pagination

Page navigation with previous/next and page numbers.

**Context:** `PaginationContext` – `page`, `set_page`, `total_pages`.

**Components:** `Pagination`, `PaginationContent`, `PaginationItem`, `PaginationLink`, `PaginationPrevious`, `PaginationNext`, `PaginationEllipsis`.

### Pagination

```rust
pub struct PaginationProps {
    #[props(default = 1)] pub page: usize,
    #[props(default = 1)] pub total_pages: usize,
    #[props(default)] pub on_change: Option<Callback<usize>>,
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### PaginationContent

```rust
pub struct PaginationContentProps { pub children: Element }
```

### PaginationItem

```rust
pub struct PaginationItemProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub active: bool,
    #[props(default = false)] pub disabled: bool,
    pub children: Element,
}
```

### PaginationLink

```rust
pub struct PaginationLinkProps {
    #[props(into, default)] pub class: String,
    #[props(default)] pub href: Option<String>,
    #[props(default = false)] pub active: bool,
    pub children: Element,
}
```

### PaginationPrevious/Next

```rust
pub struct PaginationPreviousProps {
    #[props(into, default)] pub class: String,
}
// same for PaginationNextProps
```

### PaginationEllipsis

```rust
pub struct PaginationEllipsisProps {
    #[props(into, default)] pub class: String,
}
```

**Example:**

```rust
rsx! {
    Pagination { total_pages: 10,
        PaginationContent {
            PaginationPrevious {}
            PaginationItem { active: true, PaginationLink { href: "#", "1" } }
            PaginationEllipsis {}
            PaginationNext {}
        }
    }
}
```

---

## Popover

Click‑triggered floating panel.

**Context:** `PopoverContext` – `open`, `set_open`.

**Components:** `Popover`, `PopoverTrigger`, `PopoverContent`.

### Popover

```rust
pub struct PopoverProps { pub children: Element }
```

### PopoverTrigger

```rust
pub struct PopoverTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### PopoverContent

```rust
pub struct PopoverContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = "bottom".to_string())] pub side: String,
    #[props(default = false)] pub force_mount: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Popover {
        PopoverTrigger { "Click" }
        PopoverContent { side: "bottom", "Content" }
    }
}
```

---

## Progress

Linear progress indicator.

```rust
pub struct ProgressProps {
    #[props(default)] pub value: Option<f64>, // None = indeterminate
    #[props(default = 100.0)] pub max: f64,
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Progress { value: Some(33.0) }
    Progress { value: None } // indeterminate
}
```

---

## Radio Group

A group of mutually exclusive radio buttons.

**Context:** `RadioGroupContext` – `value`, `set_value`.

**Components:** `RadioGroup`, `RadioGroupItem`, `RadioGroupLabel`.

### RadioGroup

```rust
pub struct RadioGroupProps {
    #[props(default)] pub default_value: Option<String>,
    #[props(into, default)] pub class: String,
    #[props(default = "radiogroup".to_string())] pub name: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### RadioGroupItem

```rust
pub struct RadioGroupItemProps {
    #[props(into)] pub value: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### RadioGroupLabel

```rust
pub struct RadioGroupLabelProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    RadioGroup { default_value: Some("apple".into()),
        RadioGroupItem { value: "apple" }
        RadioGroupLabel { "Apple" }
        RadioGroupItem { value: "orange" }
        RadioGroupLabel { "Orange" }
    }
}
```

---

## Resizable

Two‑panel resizable layout.

**Context:** `ResizeState` – `sizes: Signal<Vec<f64>>`.

**Components:** `ResizablePanelGroup`, `ResizablePanel`, `ResizableHandle`.

### ResizablePanelGroup

Direction: `Horizontal` or `Vertical`.

```rust
pub struct ResizablePanelGroupProps {
    #[props(default)] pub direction: Direction,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ResizablePanel

```rust
pub struct ResizablePanelProps {
    #[props(default = 0)] pub index: usize, // 0 or 1
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### ResizableHandle

```rust
pub struct ResizableHandleProps {
    #[props(default = "vertical".to_string())] pub orientation: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
}
```

**Example:**

```rust
rsx! {
    ResizablePanelGroup { direction: Direction::Horizontal,
        ResizablePanel { index: 0, "Left" }
        ResizableHandle {}
        ResizablePanel { index: 1, "Right" }
    }
}
```

---

## Scroll Area

Custom‑styled scrollable container.

**Components:** `ScrollArea`, `ScrollBar`, `ScrollBarThumb` (low‑level).

### ScrollArea

```rust
pub struct ScrollAreaProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ScrollBar

```rust
pub struct ScrollBarProps {
    #[props(into, default)] pub class: String,
    #[props(default = "vertical".to_string())] pub orientation: String,
    pub children: Element,
}
```

### ScrollBarThumb

```rust
pub struct ScrollBarThumbProps {
    #[props(into, default)] pub class: String,
}
```

**Example:**

```rust
rsx! {
    ScrollArea { class: "h-[200px] w-[350px] border", "Content..." }
}
```

---

## Select

Custom dropdown select.

**Context:** `SelectContext` – `open`, `set_open`, `value`, `set_value`.

**Components:** `Select`, `SelectTrigger`, `SelectValue`, `SelectContent`, `SelectItem`, `SelectLabel`, `SelectSeparator`.

### Select

```rust
pub struct SelectProps {
    #[props(default)] pub default_value: String,
    pub children: Element,
}
```

### SelectTrigger

```rust
pub struct SelectTriggerProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### SelectValue

```rust
pub struct SelectValueProps {
    #[props(default = "".to_string())] pub placeholder: String,
    pub children: Element,
}
```

### SelectContent

```rust
pub struct SelectContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = "bottom".to_string())] pub side: String,
    #[props(default = false)] pub force_mount: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### SelectItem

```rust
pub struct SelectItemProps {
    #[props(into)] pub value: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### SelectLabel

```rust
pub struct SelectLabelProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
}
```

### SelectSeparator

```rust
pub struct SelectSeparatorProps {
    #[props(into, default)] pub class: String,
}
```

**Example:**

```rust
rsx! {
    Select { default_value: "apple".into(),
        SelectTrigger {
            SelectValue { placeholder: "Fruit" }
        }
        SelectContent {
            SelectItem { value: "apple", "Apple" }
            SelectItem { value: "banana", "Banana" }
        }
    }
}
```

---

## Separator

Visual divider.

```rust
pub struct SeparatorProps {
    #[props(into, default)] pub class: String,
    #[props(default)] pub orientation: SeparatorOrientation, // Vertical or Horizontal
    #[props(default = false)] pub decorative: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

```rust
rsx! {
    Separator { orientation: SeparatorOrientation::Horizontal }
}
```

---

## Sheet

Side panel overlay (uses `Dialog` internally). Same API as Dialog but with `SheetContent` accepting a `side` prop.

**Re‑exports:**
- `Sheet` (alias for `Dialog`)
- `SheetTrigger`, `SheetPortal`, `SheetOverlay`, `SheetContent`, `SheetHeader`, `SheetTitle`, `SheetDescription`, `SheetFooter`, `SheetClose`

### SheetContent

```rust
pub struct SheetContentProps {
    #[props(default)] pub side: Side, // Top, Bottom, Left, Right
    #[props(into, default)] pub class: String,
    #[props(default = true)] pub show_close_button: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Sheet {
        SheetTrigger { "Open" }
        SheetPortal {
            SheetOverlay {}
            SheetContent { side: Side::Right,
                SheetHeader { SheetTitle { "Settings" } }
            }
        }
    }
}
```

---

## Sidebar

Application sidebar with collapsible support.

**Context:** `SidebarContext` (provided by `SidebarProvider`), `SidebarGroupContext`.

**Components:** `SidebarProvider`, `Sidebar`, `SidebarContent`, `SidebarHeader`, `SidebarFooter`, `SidebarGroup`, `SidebarGroupLabel`, `SidebarGroupContent`, `SidebarMenu`, `SidebarMenuItem`, `SidebarInput`, `SidebarTrigger`, `SidebarRail`, `SidebarSeparator`, `SidebarInset`, `SidebarMobileOverlay`.

### SidebarProvider

```rust
pub struct SidebarProviderProps {
    #[props(default)] pub default_open: bool,
    #[props(default)] pub variant: SidebarVariant, // Sidebar, Floating, Inset
    pub children: Element,
}
```

### Sidebar

```rust
pub struct SidebarProps {
    #[props(default)] pub collapsible: bool,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### SidebarTrigger

```rust
pub struct SidebarTriggerProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub close: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### SidebarMenu

```rust
pub struct SidebarMenuProps { ... } // class, children
```

### SidebarMenuItem

```rust
pub struct SidebarMenuItemProps {
    #[props(default = false)] pub active: bool,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    SidebarProvider { default_open: true,
        Sidebar {
            SidebarHeader { "App" }
            SidebarContent {
                SidebarGroup {
                    SidebarGroupLabel { "Main" }
                    SidebarGroupContent {
                        SidebarMenu {
                            SidebarMenuItem { active: true, "Dashboard" }
                        }
                    }
                }
            }
        }
        SidebarInset {
            // main content
        }
    }
}
```

---

## Skeleton

Placeholder for loading content.

```rust
pub struct SkeletonProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Skeleton { class: "h-4 w-[250px] rounded-full" }
}
```

---

## Slider

Range input styled as a slider.

```rust
pub struct SliderProps {
    #[props(default = 0.0)] pub min: f64,
    #[props(default = 100.0)] pub max: f64,
    #[props(default = 1.0)] pub step: f64,
    #[props(default = 0.0)] pub value: f64,
    #[props(default)] pub on_change: Option<Callback<f64>>,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

```rust
rsx! {
    Slider { value: 50.0, on_change: move |v| {} }
}
```

---

## Spinner

Loading spinner (uses `lucide-dioxus` `LoaderCircle`).

```rust
pub struct SpinnerProps {
    #[props(into, default)] pub class: String,
    #[props(default = 24)] pub size: usize,
    #[props(default = "currentColor".to_owned())] pub color: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

```rust
rsx! {
    Spinner { class: "text-blue-500" }
}
```

---

## Stepper

Multi‑step process indicator.

**Context:** `StepperContext` – `current_step`, `total_steps`, `set_step`.

**Components:** `Stepper`, `StepperItem`, `StepperTitle`, `StepperDescription`, `StepperSeparator`, `StepperFooter`, `StepperPrevious`, `StepperNext`, `StepperIndicator`.

### Stepper

```rust
pub struct StepperProps {
    #[props(default = 0)] pub default_step: usize,
    pub total_steps: usize,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### StepperItem

```rust
pub struct StepperItemProps {
    #[props(default = 0)] pub step: usize,
    #[props(default)] pub completed: bool,
    #[props(default = false)] pub disabled: bool,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### StepperTitle, StepperDescription

```rust
pub struct StepperTitleProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
// similar for StepperDescriptionProps
```

### StepperSeparator

```rust
pub struct StepperSeparatorProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### StepperFooter

```rust
pub struct StepperFooterProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### StepperPrevious, StepperNext

```rust
pub struct StepperPreviousProps {
    #[props(default = "Back".to_string())] pub label: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub onclick: Option<EventHandler<MouseEvent>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
// similar for StepperNextProps
```

### StepperIndicator

```rust
pub struct StepperIndicatorProps {
    #[props(into, default)] pub class: String,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Stepper { total_steps: 3,
        StepperItem { step: 0,
            StepperTitle { "Step 1" }
        }
        StepperSeparator {}
        StepperItem { step: 1,
            StepperTitle { "Step 2" }
        }
        StepperFooter {
            StepperPrevious {}
            StepperNext {}
        }
    }
}
```

---

## Switch

Toggle switch.

```rust
pub struct SwitchProps {
    #[props(default)] pub checked: Option<Signal<bool>>,
    #[props(default = false)] pub default_checked: bool,
    #[props(default = false)] pub disabled: bool,
    #[props(into, default)] pub class: String,
    #[props(default)] pub on_checked_change: Option<Callback<bool>>,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Switch { on_checked_change: move |checked| {} }
}
```

---

## Table

Semantic table components.

**Components:** `Table`, `TableHeader`, `TableHead`, `TableBody`, `TableRow`, `TableCell`, `TableFooter`, `TableCaption`.

### Table

```rust
pub struct TableProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableHeader

```rust
pub struct TableHeaderProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableHead

```rust
pub struct TableHeadProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableBody

```rust
pub struct TableBodyProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableRow

```rust
pub struct TableRowProps {
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub header: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableCell

```rust
pub struct TableCellProps {
    #[props(into, default)] pub class: String,
    #[props(default = 1)] pub column_span: u32,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableFooter

```rust
pub struct TableFooterProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TableCaption

```rust
pub struct TableCaptionProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Table {
        TableCaption { "Users" }
        TableHeader {
            TableRow {
                TableHead { "Name" }
            }
        }
        TableBody {
            TableRow {
                TableCell { "Alice" }
            }
        }
    }
}
```

---

## Tabs

Tabbed interface.

**Context:** `TabsContext` – `value`, `set_value`.

**Components:** `Tabs`, `TabsList`, `TabsTrigger`, `TabsContent`.

### Tabs

```rust
pub struct TabsProps {
    #[props(into, default)] pub default_value: String,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TabsList

```rust
pub struct TabsListProps {
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TabsTrigger

```rust
pub struct TabsTriggerProps {
    #[props(into)] pub value: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TabsContent

```rust
pub struct TabsContentProps {
    #[props(into)] pub value: String,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Tabs { default_value: "tab1".into(),
        TabsList {
            TabsTrigger { value: "tab1", "First" }
            TabsTrigger { value: "tab2", "Second" }
        }
        TabsContent { value: "tab1", "Content 1" }
        TabsContent { value: "tab2", "Content 2" }
    }
}
```

---

## Textarea

```rust
pub struct TextareaProps {
    #[props(into, default)] pub class: String,
    #[props(into, default = "textarea".to_string())] pub data_slot: String,
    pub onmounted: Option<EventHandler<MountedEvent>>,
    #[props(extends = GlobalAttributes)]
    #[props(extends = textarea)]
    pub attributes: Vec<Attribute>,
}
```

```rust
rsx! {
    Textarea { placeholder: "Enter text..." }
}
```

---

## Toast

Notification system.

**Global API:** `add_toast(title, description, variant, duration)`, `remove_toast(id)`, `clear_toasts()`.  
`ToastProvider` must be wrapped around your app.

**Components (internal, but can be used directly):** `Toast`, `ToastClose`, `ToastTitle`, `ToastDescription`, `ToastAction`.

### ToastProvider

```rust
pub struct ToastProviderProps { pub children: Element }
```

### Toast

```rust
pub struct ToastProps {
    pub toast: Signal<ToastData>,
}
```

`ToastData` contains `id`, `title`, `description`, `variant` (`Default`, `Destructive`, `Success`), `action`, `duration`.

### Adding toasts

```rust
add_toast("Success".to_string(), Some("Operation completed".to_string()), ToastVariant::Success, 5000);
```

**Usage:**

```rust
rsx! {
    ToastProvider {
        // ... your app
        // toasts will show at bottom-right
    }
}
```

---

## Toggle

Two‑state toggle button.

Variants: `Default`, `Outline`. Sizes: `Default`, `Sm`, `Lg`.

```rust
pub struct ToggleProps {
    #[props(default)] pub pressed: Option<Signal<bool>>,
    #[props(default = false)] pub default_pressed: bool,
    #[props(default = false)] pub disabled: bool,
    #[props(default)] pub variant: ToggleVariant,
    #[props(default)] pub size: ToggleSize,
    #[props(into, default)] pub class: String,
    #[props(default)] pub on_pressed_change: Option<Callback<bool>>,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

Helper: `toggle_variants(variant, size) -> String`.

```rust
rsx! {
    Toggle { variant: ToggleVariant::Outline, "Bold" }
}
```

---

## Toggle Group

Group of toggle buttons (single or multiple selection).

**Context:** `ToggleGroupContext` – `value`, `set_value`, `group_type`.

**Components:** `ToggleGroup`, `ToggleGroupItem`.

### ToggleGroup

Type: `Single` or `Multiple`.

```rust
pub struct ToggleGroupProps {
    #[props(default)] pub group_type: ToggleGroupType,
    #[props(default)] pub default_value: Vec<String>,
    #[props(into, default)] pub class: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### ToggleGroupItem

```rust
pub struct ToggleGroupItemProps {
    #[props(into)] pub value: String,
    #[props(into, default)] pub class: String,
    #[props(default = false)] pub disabled: bool,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    ToggleGroup { group_type: ToggleGroupType::Single,
        ToggleGroupItem { value: "a", "A" }
        ToggleGroupItem { value: "b", "B" }
    }
}
```

---

## Tooltip

Hover/focus tooltip.

**Context:** `TooltipContext` – `open`, `set_open`.

**Components:** `Tooltip`, `TooltipTrigger`, `TooltipContent`.

### Tooltip

```rust
pub struct TooltipProps {
    #[props(default = 0)] pub delay_duration: u64,
    pub children: Element,
}
```

### TooltipTrigger

```rust
pub struct TooltipTriggerProps {
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

### TooltipContent

```rust
pub struct TooltipContentProps {
    #[props(into, default)] pub class: String,
    #[props(default = "top".to_string())] pub side: String,
    pub children: Element,
    #[props(extends = GlobalAttributes)] pub attributes: Vec<Attribute>,
}
```

**Example:**

```rust
rsx! {
    Tooltip {
        TooltipTrigger { button { "Hover" } }
        TooltipContent { side: "top", "Tooltip text" }
    }
}
```

---

## Utilities

- `cn(base: &str, additional: &str) -> String` – merge Tailwind classes safely using `tailwind_fuse`.
- `RenderFn` – used with `Item`’s `as_child` prop to render as a custom element (e.g., `<a>`). Construct with `RenderFn::new(fn)`.
- `PortalProvider` – must wrap the root of your app for dialogs, sheets, etc.

---

This covers the entire publicly exported API of the `ui` crate. For all components, refer to the source docstrings and test files for additional examples. When in doubt, check the corresponding `tests/*.rs` files for usage patterns.
