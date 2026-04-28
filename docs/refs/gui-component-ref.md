Below is the complete API usage documentation for all components of the `gpui-component` library, extracted from the repository’s own guide and source files. It covers every component listed in the official gallery and explains how to use each one with code examples that can be directly copied.

---

# GPUI Component – Full API Usage Guide

## Table of Contents

1. [Getting Started](#getting-started)
2. [Core Concepts](#core-concepts)
   - [Root View](#root-view)
   - [Contexts](#contexts)
   - [ElementId](#elementid)
   - [Theme & Assets](#theme--assets)
3. Components
   - [Accordion](#accordion)
   - [Alert](#alert)
   - [AlertDialog](#alertdialog)
   - [Avatar](#avatar)
   - [Badge](#badge)
   - [Breadcrumb](#breadcrumb)
   - [Button](#button)
   - [Calendar](#calendar)
   - [Chart](#chart)
   - [Checkbox](#checkbox)
   - [Clipboard](#clipboard)
   - [Collapsible](#collapsible)
   - [ColorPicker](#colorpicker)
   - [DataTable](#datatable)
   - [DatePicker](#datepicker)
   - [DescriptionList](#descriptionlist)
   - [Dialog](#dialog)
   - [DropdownButton](#dropdownbutton)
   - [Editor](#editor)
   - [FocusTrap](#focustrap)
   - [Form](#form)
   - [GroupBox](#groupbox)
   - [HoverCard](#hovercard)
   - [Icon](#icon)
   - [Image](#image)
   - [Input](#input)
   - [Kbd](#kbd)
   - [Label](#label)
   - [List](#list)
   - [Menu](#menu)
   - [Notification](#notification)
   - [NumberInput](#numberinput)
   - [OtpInput](#otpinput)
   - [Pagination](#pagination)
   - [Plot](#plot)
   - [Popover](#popover)
   - [Progress](#progress)
   - [Radio](#radio)
   - [Rating](#rating)
   - [Resizable](#resizable)
   - [Scrollable](#scrollable)
   - [Select](#select)
   - [Settings](#settings)
   - [Sheet](#sheet)
   - [Sidebar](#sidebar)
   - [Skeleton](#skeleton)
   - [Slider](#slider)
   - [Spinner](#spinner)
   - [Stepper](#stepper)
   - [Switch](#switch)
   - [Table](#table)
   - [Tabs](#tabs)
   - [Tag](#tag)
   - [TitleBar](#titlebar)
   - [Toggle](#toggle)
   - [Tooltip](#tooltip)
   - [Tree](#tree)
   - [VirtualList](#virtuallist)

---

## Getting Started

Add the following to `Cargo.toml`:

```toml
gpui = { git = "https://github.com/zed-industries/zed" }
gpui_platform = { git = "https://github.com/zed-industries/zed", features = ["font-kit"] }
gpui-component = { git = "https://github.com/longbridge/gpui-component" }
gpui-component-assets = { git = "https://github.com/longbridge/gpui-component" }  # optional, for default icons
```

Initialize the library and create a window:

```rust
gpui_platform::application().with_assets(gpui_component_assets::Assets).run(move |cx| {
    gpui_component::init(cx);
    cx.spawn(async move |cx| {
        cx.open_window(WindowOptions::default(), |window, cx| {
            cx.new(|cx| Root::new(view, window, cx))
        }).expect("failed");
    }).detach();
});
```

---

## Core Concepts

### Root View

The **first** view in every window **must** be `Root`. It handles:
- Dialogs (`Root::render_dialog_layer`)
- Sheets (`Root::render_sheet_layer`)
- Notifications (`Root::render_notification_layer`)
- Keyboard focus trap and tab navigation.

Example rendering these layers in your top-level view:

```rust
impl Render for MyApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .child(/* your content */)
            .children(Root::render_sheet_layer(window, cx))
            .children(Root::render_dialog_layer(window, cx))
            .children(Root::render_notification_layer(window, cx))
    }
}
```

### Contexts

- `Window` – window-level operations.
- `App` – application-level operations.
- `Context<T>` – entity context, used for stateful components.
- `Entity<T>` – an owned handle to a stateful component.

### ElementId

Every element can have an `id(...)`. It must be **unique within the same parent**. This is used for internal state management and event dispatch.

### Theme & Assets

Access theme colors via `ActiveTheme`:

```rust
use gpui_component::ActiveTheme as _;
cx.theme().primary
```

Icons require an asset source. The default one is `gpui_component_assets::Assets`. To use custom icons, implement `AssetSource` and pass it to `.with_assets(…)`.

---

## Component Documentation

### Accordion

Collapsible content panels with optional multi-open and borders.

```rust
Accordion::new("my-accordion")
    .multiple(true)
    .item(|item| item.title("Section 1").child("Content 1"))
    .item(|item| item.title("Section 2").child("Content 2"))
    .on_toggle_click(|open_indices, _, _| println!("{:?}", open_indices));
```

Sizes: `.large()`, `.small()`, `.xsmall()` (default medium).

### Alert

Callout messages with variants: info, success, warning, error.

```rust
Alert::info("alert-id", "Informational message").title("Info")
Alert::success("id", "Success message").banner()
Alert::warning("id", markdown("- **Warning**")).on_close(|_,_,_| {})
```

Closable with `on_close()`. Banner mode takes full width.

### AlertDialog

A dedicated confirmation dialog built on `Dialog`.

```rust
// Imperative API
window.open_alert_dialog(cx, |alert, _, _| {
    alert.title("Delete?").description("This cannot be undone.").show_cancel(true)
});

// Declarative API (trigger + content)
AlertDialog::new(cx)
    .trigger(Button::new("open").label("Show"))
    .content(|content, _, _| {
        content.child(DialogHeader::new().child(DialogTitle::new().child("Confirm")))
            .child(DialogFooter::new().child(Button::new("ok").primary().label("OK")))
    })
```

Use `DialogAction` / `DialogClose` wrappers for automatic callback handling.

### Avatar

User avatar with image or initials fallback.

```rust
Avatar::new().src("https://...").name("John Doe").large()
Avatar::new().name("John Doe")             // initials
AvatarGroup::new().limit(3).ellipsis()
    .child(Avatar::new().src("..."))
```

Sizes: `.xsmall()`, `.small()`, `.medium()`, `.large()`, or custom with `.with_size(px(n))`.

### Badge

Count, dot, or icon badge on an element.

```rust
Badge::new().count(5).child(Icon::new(IconName::Bell))
Badge::new().dot().color(cx.theme().green).child(Avatar::new())
Badge::new().icon(IconName::Check).child(Avatar::new())
```

### Breadcrumb

Navigation breadcrumbs.

```rust
Breadcrumb::new()
    .child("Home")
    .child(BreadcrumbItem::new("Documents").on_click(|_,_,_| {}))
    .child("Project")
```

### Button

Many variants: primary, secondary, danger, ghost, link, text. Supports icons, loading, outline, compact, and dropdown caret.

```rust
Button::new("btn").primary().label("Primary").icon(IconName::Check).on_click(|_,_,_|{})
Button::new("btn").outline().danger().label("Danger")
Button::new("btn").ghost().icon(IconName::Search)
ButtonGroup::new("grp").child(Button::new(0).label("A")).child(Button::new(1).label("B"))
ToggleGroup / Toggle analogous.
```

### Calendar

Standalone date display supporting single/range selection, disabled matchers, and multiple months.

```rust
let state = cx.new(|cx| CalendarState::new(window, cx).set_date(Local::now().date(), window, cx));
Calendar::new(&state).number_of_months(2)

// Disable weekends:
state.update(cx, |s, _| s.disabled_matcher = Some(Matcher::DayOfWeek(vec![0,6])));
```

Events: `CalendarEvent::Selected(Date)`.

### Chart

Line, bar, area, pie, candlestick charts.

```rust
LineChart::new(data)
    .x(|d| d.time.clone()).y(|d| d.value).stroke(cx.theme().chart_1).dot()

BarChart::new(data)
    .x(|d| d.category.clone()).y(|d| d.value).fill(|d| d.color).label(|d| d.value)

PieChart::new(data).value(|d| d.amount).outer_radius(100.).inner_radius(60.)

AreaChart::new(data).x(|d| ...).y(|d| ...).fill(linear_gradient(…))

CandlestickChart::new(data).x(|d| d.date).open(|d| d.open).high(|d| d.high).low(|d| d.low).close(|d| d.close)
```

All accept optional `tick_margin`, grid, and axis customisation.

### Checkbox

Binary selection.

```rust
Checkbox::new("chk").label("Accept").checked(true).on_click(|checked,_,_|{})
Checkbox::new("chk").disabled(true)
```

### Clipboard

Copy button with feedback.

```rust
Clipboard::new("cp").value("text to copy").on_copied(|_,_,_|{})
Clipboard::new("cp").value_fn(|_,cx| compute_value(cx))
```

### Collapsible

Expand/collapse content section.

```rust
Collapsible::new().open(self.is_open)
    .child("Header")
    .content("Hidden body")
```

### ColorPicker

Color selection with palette, HSLA sliders, hex input.

```rust
let state = cx.new(|cx| ColorPickerState::new(window, cx).default_value(cx.theme().primary));
ColorPicker::new(&state).label("Colour")
cx.subscribe(&state, |this, _, ev, _| match ev { ColorPickerEvent::Change(c)=>{} });
```

### DataTable

High‑performance table with virtual rows & columns, sorting, column reorder, infinite loading.

```rust
let state = cx.new(|cx| TableState::new(MyDelegate::new(), window, cx));
DataTable::new(&state).stripe(true)

// Delegate must implement TableDelegate trait
impl TableDelegate for MyDelegate { … }
```

### DatePicker

Inline or popup date selection (single/range).

```rust
let state = cx.new(|cx| DatePickerState::new(window, cx));
DatePicker::new(&state).cleanable(true).placeholder("Select date")
```

### DescriptionList

Key‑value pairs in grid layout.

```rust
DescriptionList::new()
    .item("Name", "GPUI Component", 1)
    .item("Description", "A library", 2)
    .bordered(false)
```

### Dialog

General‑purpose modal dialog.

```rust
window.open_dialog(cx, |dialog, _, _| {
    dialog.title("Dialog").child(Input::new(&input)).footer(…)
})

// Declarative API:
Dialog::new(cx).trigger(Button::new("open").label("Open"))
    .content(|content,_,_| content.child(DialogHeader::new()…))
```

### DropdownButton

Button with attached dropdown menu.

```rust
DropdownButton::new("btn").primary()
    .button(Button::new("btn").label("Actions"))
    .dropdown_menu(|menu,_,_| menu.menu("Action",Box::new(Action)))
```

### Editor

Multi‑line text with optional code editor (syntax highlighting via tree‑sitter, LSP).

```rust
let state = cx.new(|cx| InputState::new(window, cx).code_editor("rust").line_number(true));
Input::new(&state).h_full()
```

### FocusTrap

Traps keyboard focus within a container. Automatically used in Dialog and Sheet.

```rust
v_flex().child(…).focus_trap("trap-id", &focus_handle)
```

### Form

Structured form with vertical/horizontal layout.

```rust
v_form()
    .child(field().label("Name").child(Input::new(&name)))
    .child(field().label("Email").required(true).child(Input::new(&email)))
```

### GroupBox

Styled container with optional title.

```rust
GroupBox::new().fill().title("Settings").child(…)
GroupBox::new().outline().title("Preferences").child(…)
```

### HoverCard

Tooltip‑like card that appears on hover.

```rust
HoverCard::new("card").trigger(Button::new("hover").label("Hover"))
    .child(div().child("Rich content"))
    .open_delay(Duration::from_millis(500))
```

### Icon

SVG icon (requires assets).

```rust
Icon::new(IconName::Heart).small().text_color(cx.theme().red)
// Directly use IconName in many places
Button::new("btn").icon(IconName::Check)
```

### Image

Basic image display.

```rust
img("url").w(px(300.)).h(px(200.)).object_fit(ObjectFit::Cover)
```

### Input

Single‑line text input with validation, masking, prefix/suffix.

```rust
let state = cx.new(|cx| InputState::new(window, cx));
Input::new(&state).cleanable(true).placeholder("Enter text")
```

### Kbd

Keyboard shortcut display.

```rust
Kbd::new(Keystroke::parse("cmd-shift-p").unwrap())     // renders ⌘⇧P on macOS
```

### Label

Text with optional secondary text, highlights, masking.

```rust
Label::new("Name").secondary("(optional)").highlights("Na")
Label::new("...").masked(true)
```

### List

General‑purpose scrollable list with sections, search, and infinite loading.

```rust
let delegate = MyListDelegate::new();
let state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));
List::new(&state)
```

### Menu

Popup / context menus with icons, shortcuts, submenus.

```rust
Button::new("menu").dropdown_menu(|menu,_,_| {
    menu.menu("Copy", Box::new(Copy)).separator().menu("Paste", Box::new(Paste))
})

div().context_menu(|menu,_,_| menu.menu("Action", Box::new(MyAction)))
```

### Notification

Toast notifications, auto‑dismiss, actions.

```rust
window.push_notification("Saved", cx);
Notification::success("Upload complete").title("Success").autohide(false)
```

### NumberInput

Numeric input with stepper buttons.

```rust
let state = cx.new(|cx| InputState::new(window, cx).default_value("0"));
NumberInput::new(&state).prefix("$")
```

### OtpInput

One‑time password input.

```rust
let state = cx.new(|cx| OtpState::new(6, window, cx).masked(true));
OtpInput::new(&state).groups(2)
```

### Pagination

Page navigation.

```rust
Pagination::new("pgr").current_page(5).total_pages(10).on_click(|page,_,_| {})
Pagination::new("pgr").compact().visible_pages(7)
```

### Plot

Low‑level charting primitives (scales, axes, shapes).

```rust
let scale = ScaleLinear::new(vec![0., 100.], vec![0., 500.]);
PlotAxis::new().x(height).x_label(labels).paint(&bounds, window, cx);
```

### Popover

Floating content anchored to a trigger.

```rust
Popover::new("popover").trigger(Button::new("btn").label("Click"))
    .anchor(Anchor::TopRight).child("Popover content")
```

### Progress

Linear progress bar and circular progress indicator.

```rust
Progress::new("bar").value(75.).loading(false)
ProgressCircle::new("circle").value(50.).color(cx.theme().green)
```

### Radio

Single selection from multiple options.

```rust
Radio::new("r1").label("Option 1").checked(true).on_click(|checked,_,_|{})
RadioGroup::horizontal("group").children(["A","B"]).selected_index(Some(0))
```

### Rating

Star rating.

```rust
Rating::new("rate").value(3).max(5).on_click(|value,_,_|{})
Rating::new("rate").large().color(cx.theme().green)
```

### Resizable

Draggable split panels.

```rust
h_resizable("layout")
    .child(resizable_panel().size(px(200.)).child(…))
    .child(resizable_panel().child(…))
```

### Scrollable

Add scrollbars to any container.

```rust
div().overflow_scrollbar()   // or .vertical_scrollbar(handle)
```

### Select

Dropdown selection.

```rust
let items = SearchableVec::new(vec!["Rust","Go","Python"]);
let state = cx.new(|cx| SelectState::new(items, None, window, cx));
Select::new(&state).icon(IconName::Search)
```

### Settings

Settings page with search, grouping, various field types.

```rust
Settings::new("app-settings").pages(vec![
    SettingPage::new("General").group(
        SettingGroup::new().title("Appearance").item(
            SettingItem::new("Dark Mode", SettingField::switch(|cx|…, |val,cx|…))
        )
    )
])
```

### Sheet

Slide‑in panel from any edge.

```rust
window.open_sheet_at(Placement::Left, cx, |sheet,_,_| {
    sheet.title("Panel").size(px(300.)).child(content)
})
```

### Sidebar

App navigation sidebar with collapse, nested items.

```rust
Sidebar::new().collapsible(true).collapsed(false)
    .header(SidebarHeader::new().child("App Name"))
    .child(SidebarGroup::new("Nav").child(
        SidebarMenu::new().child(
            SidebarMenuItem::new("Dashboard").icon(IconName::Home).active(true)
        )
    ))
```

### Skeleton

Placeholder loading animation.

```rust
Skeleton::new().w(px(200.)).h_4().rounded_md()
```

### Slider

Range slider with thumb dragging.

```rust
let state = cx.new(|_| SliderState::new().min(0.).max(100.).default_value(50.));
Slider::new(&state).vertical().h(px(200.))
```

### Spinner

Rotating loading indicator.

```rust
Spinner::new().color(cx.theme().primary).large()
```

### Stepper

Step‑by‑step indicator.

```rust
Stepper::new("stepper").selected_index(0)
    .items([StepperItem::new().child("Step 1"), …])
```

### Switch

Toggle switch (on/off).

```rust
Switch::new("sw").label("Enable").checked(true).on_click(|checked,_,_|{})
```

### Table

Simple, stateless table component.

```rust
Table::new()
    .child(TableHeader::new().child(TableRow::new().child(TableHead::new().child("Name"))))
    .child(TableBody::new().child(TableRow::new().child(TableCell::new().child("John"))))
```

### Tabs

Tab bar with multiple styles.

```rust
TabBar::new("tabs").selected_index(0)
    .child(Tab::new().label("Account"))
    .child(Tab::new().label("Settings"))
```

### Tag

Category labels.

```rust
Tag::primary().child("New")
Tag::danger().outline().small().child("Critical")
```

### TitleBar

Custom window title bar with menu integration.

```rust
TitleBar::new().child(div().child("My App"))
    .child(Button::new("github").icon(IconName::GitHub))
```

### Toggle

Button‑style toggle (pressed/not pressed).

```rust
Toggle::new("toggle").label("Bold").checked(self.bold).on_click(|checked,_,_|{})
```

### Tooltip

Hover‑activated rich tooltip.

```rust
Button::new("btn").tooltip("Delete this file")
// or
div().tooltip(|window,cx| Tooltip::new("Rich content").build(window, cx))
```

### Tree

Hierarchical tree view.

```rust
let state = cx.new(|cx| TreeState::new(cx).items(vec![TreeItem::new("src","src").child(…)…]));
tree(&state, |ix, entry, selected, _, cx| {
    ListItem::new(ix).child(entry.item().label.clone())
})
```

### VirtualList

High‑performance list for very large datasets with variable item sizes.

```rust
v_virtual_list(cx.entity(), "list", item_sizes.clone(),
    |view, visible_range, _, cx| { visible_range.map(|ix| render_item(ix)).collect() })
```

---

All components can be styled using the standard `Styled` trait (`bg()`, `rounded()`, `p_…`) and many inherit `Sizable` for sizes. For further details and exhaustive API reference, visit the repository’s documentation or source files.
