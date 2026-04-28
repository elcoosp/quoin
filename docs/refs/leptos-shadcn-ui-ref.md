# Leptos ShadCN UI Umbrella Crate – Complete API Reference

The umbrella crate `leptos-shadcn-ui` re-exports all ShadCN components from the individual `leptos-shadcn-*` packages.  
Add it to your `Cargo.toml`:

```toml
leptos-shadcn-ui = { version = "0.9.1", features = ["all-components"] }
```

Then import everything you need from `leptos_shadcn_ui::*` or from specific modules like `leptos_shadcn_ui::button::Button`.  
All components support both **Default** and **New York** theme variants (suffix `NewYork`).  
Signal‑managed variants are available with the prefix `SignalManaged` or `Enhanced`; they offer more granular control and performance monitoring, but the standard versions are sufficient for most use cases.

---

## Table of Contents
- [Accordion](#accordion)
- [Alert](#alert)
- [Alert Dialog](#alert-dialog)
- [Avatar](#avatar)
- [Badge](#badge)
- [Breadcrumb](#breadcrumb)
- [Button](#button)
- [Calendar](#calendar)
- [Card](#card)
- [Carousel](#carousel)
- [Checkbox](#checkbox)
- [Collapsible](#collapsible)
- [Combobox](#combobox)
- [Command](#command)
- [Context Menu](#context-menu)
- [Date Picker](#date-picker)
- [Dialog](#dialog)
- [Drawer](#drawer)
- [Dropdown Menu](#dropdown-menu)
- [Error Boundary](#error-boundary)
- [Form](#form)
- [Hover Card](#hover-card)
- [Input](#input)
- [Input OTP](#input-otp)
- [Label](#label)
- [Lazy Loading](#lazy-loading)
- [Menubar](#menubar)
- [Navigation Menu](#navigation-menu)
- [Pagination](#pagination)
- [Popover](#popover)
- [Progress](#progress)
- [Radio Group](#radio-group)
- [Resizable](#resizable)
- [Select](#select)
- [Separator](#separator)
- [Sheet](#sheet)
- [Skeleton](#skeleton)
- [Slider](#slider)
- [Switch](#switch)
- [Table](#table)
- [Tabs](#tabs)
- [Textarea](#textarea)
- [Toast](#toast)
- [Toggle](#toggle)
- [Tooltip](#tooltip)
- [Global Types & Helpers](#global-types)

---

### Accordion
**Package:** `leptos_shadcn_accordion` → re‑exported as `leptos_shadcn_ui::accordion::*`

```rust
use leptos_shadcn_ui::accordion::{
    Accordion, AccordionItem, AccordionTrigger, AccordionContent,
    AccordionType, AccordionOrientation,
};
// New York: AccordionNewYork, AccordionItemNewYork, …
```

#### Accordion
```rust
[component]
pub fn Accordion(
    #[prop(into, optional)] r#type: Signal<AccordionType>,
    #[prop(into, optional)] orientation: Signal<AccordionOrientation>,
    #[prop(into, optional)] collapsible: Signal<bool>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into)] value: RwSignal<Vec<String>>,
    #[prop(into, optional)] default_value: Vec<String>,
    #[prop(into, optional)] on_value_change: Option<Callback<Vec<String>>>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

#### AccordionItem
```rust
[component]
pub fn AccordionItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

#### AccordionTrigger
```rust
[component]
pub fn AccordionTrigger(
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] as_child: Option<Callback<AccordionTriggerChildProps>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

#### AccordionContent
```rust
[component]
pub fn AccordionContent(
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] force_mount: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

**Enums:**
- `AccordionType`: `Single`, `Multiple`
- `AccordionOrientation`: `Vertical`, `Horizontal`

---

### Alert
**Package:** `leptos_shadcn_alert` → `leptos_shadcn_ui::alert::*`

```rust
use leptos_shadcn_ui::alert::{Alert, AlertTitle, AlertDescription, AlertVariant};
```

```rust
[component]
pub fn Alert(
    #[prop(into, optional)] variant: MaybeProp<AlertVariant>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

`AlertTitle`, `AlertDescription` – identical props but without `variant`.  
**Enum:** `AlertVariant` – `Default`, `Destructive`, `Success`, `Warning`

---

### Alert Dialog
**Package:** `leptos_shadcn_alert_dialog` → `leptos_shadcn_ui::alert_dialog::*`

```rust
use leptos_shadcn_ui::alert_dialog::{
    AlertDialog, AlertDialogTrigger, AlertDialogContent, AlertDialogOverlay,
    AlertDialogHeader, AlertDialogFooter, AlertDialogTitle, AlertDialogDescription,
    AlertDialogAction, AlertDialogCancel,
};
```

**AlertDialog**
```rust
[component]
pub fn AlertDialog(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into, optional)] on_open_change: Option<Callback<bool>>,
    children: ChildrenFn,
) -> impl IntoView
```
Sub‑components accept optional `class`, `id`, `style` and children; `AlertDialogAction`/`Cancel` close the dialog.

---

### Avatar
**Package:** `leptos_shadcn_avatar` → `leptos_shadcn_ui::avatar::*`

```rust
use leptos_shadcn_ui::avatar::{Avatar, AvatarImage, AvatarFallback, AvatarGroup};
```

```rust
[component]
pub fn Avatar(
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`AvatarImage`: requires `src: String`, optional `alt`, `class`, `id`, `style`.  
`AvatarFallback`, `AvatarGroup` – same as Avatar.

---

### Badge
**Package:** `leptos_shadcn_badge` → `leptos_shadcn_ui::badge::*`

```rust
use leptos_shadcn_ui::badge::{Badge, BadgeVariant};
```

```rust
[component]
pub fn Badge(
    #[prop(into, optional)] variant: MaybeProp<BadgeVariant>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
**Enum:** `BadgeVariant` – `Default`, `Secondary`, `Destructive`, `Outline`

---

### Breadcrumb
**Package:** `leptos_shadcn_breadcrumb` → `leptos_shadcn_ui::breadcrumb::*`

```rust
use leptos_shadcn_ui::breadcrumb::{
    Breadcrumb, BreadcrumbList, BreadcrumbItem,
    BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, BreadcrumbEllipsis,
};
```

All accept optional `class` and `children`.  
`BreadcrumbLink` adds optional `href: MaybeProp<String>`, `as_child: MaybeProp<bool>`.  
`BreadcrumbSeparator` can have custom children (default: chevron SVG).  
`BreadcrumbEllipsis` renders `…` with screen‑reader “More”.

---

### Button
**Package:** `leptos_shadcn_button` → `leptos_shadcn_ui::button::*`

```rust
use leptos_shadcn_ui::button::{
    Button, ButtonVariant, ButtonSize, ButtonResponsiveSize,
    ButtonChildProps, BUTTON_TOUCH_CLASS,
};
// New York: ButtonNewYork with same API
```

```rust
[component]
pub fn Button(
    #[prop(into, optional)] variant: MaybeProp<ButtonVariant>,
    #[prop(into, optional)] size: MaybeProp<ButtonSize>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] loading: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] aria_describedby: MaybeProp<String>,
    #[prop(into, optional)] as_child: Option<Callback<ButtonChildProps, AnyView>>,
    #[prop(optional)] children: Option<Children>,
    #[prop(into, optional)] responsive_size: MaybeProp<ButtonResponsiveSize>,
    #[prop(into, optional)] touch_friendly: Signal<bool>,
) -> impl IntoView
```

**Enums:**
- `ButtonVariant`: `Default`, `Destructive`, `Outline`, `Secondary`, `Ghost`, `Link`
- `ButtonSize`: `Default`, `Sm`, `Lg`, `Icon`
- `ButtonResponsiveSize`: `SmMd`, `MdLg`, `XsSm`

---

### Calendar
**Package:** `leptos_shadcn_calendar` → `leptos_shadcn_ui::calendar::*`

```rust
use leptos_shadcn_ui::calendar::{Calendar, CalendarDate};
```

```rust
[component]
pub fn Calendar(
    #[prop(into, optional)] mode: Signal<String>,
    #[prop(into, optional)] selected: RwSignal<Option<CalendarDate>>,
    #[prop(into, optional)] on_select: Option<Callback<CalendarDate>>,
    #[prop(into, optional)] disabled: Signal<Vec<CalendarDate>>,
    #[prop(into, optional)] initial_focus: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`CalendarDate::new(year, month, day)`

---

### Card
**Package:** `leptos_shadcn_card` → `leptos_shadcn_ui::card::*`

```rust
use leptos_shadcn_ui::card::{
    Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter,
    CardVariant, InteractiveCard,
};
```

```rust
[component]
pub fn Card(
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(into, optional)] variant: MaybeProp<CardVariant>,
    #[prop(into, optional)] interactive: Signal<bool>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`CardHeader`, `CardTitle`, … accept `class`, `id`, `style`, children.  
`InteractiveCard` adds `on_click: Option<Callback<()>>`.  
**Enum:** `CardVariant` – `Default`, `Destructive`, `Warning`, `Success`

---

### Carousel
**Package:** `leptos_shadcn_carousel` → `leptos_shadcn_ui::carousel::*`

```rust
use leptos_shadcn_ui::carousel::{
    Carousel, CarouselContent, CarouselItem,
    CarouselPrevious, CarouselNext, CarouselOrientation, CarouselApi,
};
```

```rust
[component]
pub fn Carousel(
    #[prop(into, optional)] orientation: MaybeProp<Signal<CarouselOrientation>>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`CarouselContent`, `CarouselItem` – children/class.  
`CarouselPrevious`/`Next` – optional `class`, `on_click`, children; include default icons & sr‑only labels.  
**Enum:** `CarouselOrientation` – `Horizontal`, `Vertical`

---

### Checkbox
**Package:** `leptos_shadcn_checkbox` → `leptos_shadcn_ui::checkbox::*`

```rust
use leptos_shadcn_ui::checkbox::Checkbox;
```

```rust
[component]
pub fn Checkbox(
    #[prop(into, optional)] checked: Signal<bool>,
    #[prop(into, optional)] on_change: Option<Callback<bool>>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
) -> impl IntoView
```

---

### Collapsible
**Package:** `leptos_shadcn_collapsible` → `leptos_shadcn_ui::collapsible::*`

```rust
use leptos_shadcn_ui::collapsible::{
    Collapsible, CollapsibleTrigger, CollapsibleContent,
};
```

```rust
[component]
pub fn Collapsible(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into, optional)] default_open: bool,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] on_open_change: Option<Callback<bool>>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`CollapsibleTrigger` – optional `class`, `as_child`, children; keyboard accessible.  
`CollapsibleContent` – optional `class`, `force_mount`, children.

---

### Combobox
**Package:** `leptos_shadcn_combobox` → `leptos_shadcn_ui::combobox::*`

```rust
use leptos_shadcn_ui::combobox::{Combobox, ComboboxOption};
```

```rust
[component]
pub fn Combobox(
    #[prop(into, optional)] value: MaybeProp<String>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] placeholder: MaybeProp<String>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into)] options: Vec<ComboboxOption>,
    #[prop(into, optional)] open: Signal<bool>,
    #[prop(optional)] on_open_change: Option<Callback<bool>>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] _children: Option<Children>,
) -> impl IntoView
```
`ComboboxOption::new(value, label)`; has field `disabled` settable via `.disabled(bool)`.

---

### Command
**Package:** `leptos_shadcn_command` → `leptos_shadcn_ui::command::*`

```rust
use leptos_shadcn_ui::command::{
    Command, CommandInput, CommandList, CommandEmpty,
    CommandGroup, CommandGroupHeading, CommandItem,
    CommandShortcut, CommandSeparator,
};
```

```rust
[component]
pub fn Command(
    #[prop(optional)] value: MaybeProp<String>,
    #[prop(optional)] on_value_change: Option<Callback<String>>,
    #[prop(optional)] class: MaybeProp<String>,
    children: Children,
) -> impl IntoView
```
`CommandItem` accepts optional `value: MaybeProp<String>`, `disabled: MaybeProp<bool>`, `class`, `id`, `style`, children.  
All sub‑components accept optional class/id/style.

---

### Context Menu
**Package:** `leptos_shadcn_context_menu` → `leptos_shadcn_ui::context_menu::*`

```rust
use leptos_shadcn_ui::context_menu::{
    ContextMenu, ContextMenuTrigger, ContextMenuContent,
    ContextMenuItem, ContextMenuSeparator, ContextMenuLabel,
    ContextMenuCheckboxItem, ContextMenuRadioGroup, ContextMenuRadioItem,
    ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuShortcut,
};
```

**ContextMenu** – no props, wraps children; provides `open` and `position`.  
**ContextMenuTrigger** – optional `class`, children; listens to `contextmenu` event.  
**ContextMenuContent** – optional `class`, `id`, `style`, children; positioned via `position` signal.  
**ContextMenuItem** – optional `disabled`, `class`, `id`, `style`, children.  
**ContextMenuCheckboxItem** – requires `checked: RwSignal<bool>`, optional `on_checked_change`.  
**ContextMenuRadioGroup** – requires `value: RwSignal<String>`, optional `on_value_change`.  
**ContextMenuRadioItem** – requires `value: String`.  
**ContextMenuSub** – wraps a sub‑menu with its own `open` signal.  
Others (`Separator`, `Label`, `Shortcut`) accept optional class/id/style.

---

### Date Picker
**Package:** `leptos_shadcn_date_picker` → `leptos_shadcn_ui::date_picker::*`

```rust
use leptos_shadcn_ui::date_picker::{DatePicker, DatePickerWithRange};
use leptos_shadcn_ui::calendar::CalendarDate; // required
```

```rust
[component]
pub fn DatePicker(
    #[prop(optional)] selected: MaybeProp<CalendarDate>,
    #[prop(optional)] on_select: Option<Callback<CalendarDate>>,
    #[prop(optional)] disabled: MaybeProp<Vec<CalendarDate>>,
    #[prop(optional)] placeholder: MaybeProp<String>,
    #[prop(optional)] class: MaybeProp<String>,
) -> impl IntoView
```
`DatePickerWithRange` uses `from`, `to` and `on_select(Option<(Option<CalendarDate>, Option<CalendarDate>)>)`.

---

### Dialog
**Package:** `leptos_shadcn_dialog` → `leptos_shadcn_ui::dialog::*`

```rust
use leptos_shadcn_ui::dialog::{
    Dialog, DialogTrigger, DialogContent, DialogHeader,
    DialogTitle, DialogDescription, DialogFooter, DialogClose,
};
```

```rust
[component]
pub fn Dialog(
    #[prop(into, optional)] open: Signal<bool>,
    #[prop(into, optional)] on_open_change: Option<Callback<bool>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`DialogContent` – optional `class`, `style`, children.  
`DialogClose` – click closes dialog.

---

### Drawer
**Package:** `leptos_shadcn_drawer` → `leptos_shadcn_ui::drawer::*`

```rust
use leptos_shadcn_ui::drawer::{
    Drawer, DrawerTrigger, DrawerContent, DrawerOverlay,
    DrawerPortal, DrawerHeader, DrawerFooter, DrawerTitle,
    DrawerDescription, DrawerClose, DrawerNestedRoot, DrawerDirection,
};
```

```rust
[component]
pub fn Drawer(
    #[prop(into)] open: RwSignal<bool>,
    #[prop(into, optional)] on_open_change: Option<Callback<bool>>,
    #[prop(into, optional)] direction: Signal<DrawerDirection>,
    #[prop(into, optional)] should_scale_background: Signal<bool>,
    children: ChildrenFn,
) -> impl IntoView
```
**Enum:** `DrawerDirection` – `Top`, `Bottom`, `Left`, `Right`

---

### Dropdown Menu
**Package:** `leptos_shadcn_dropdown_menu` → `leptos_shadcn_ui::dropdown_menu::*`

```rust
use leptos_shadcn_ui::dropdown_menu::DropdownMenu;
```

```rust
[component]
pub fn DropdownMenu(
    #[prop(into, optional)] variant: MaybeProp<String>,
    #[prop(into, optional)] size: MaybeProp<String>,
    #[prop(into, optional)] on_click: Option<Callback<()>>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
For full menu functionality, use the **ContextMenu** component.

---

### Error Boundary
**Package:** `leptos_shadcn_error_boundary` → `leptos_shadcn_ui::error_boundary::*`

```rust
use leptos_shadcn_ui::error_boundary::{
    ErrorBoundary, RichErrorBoundary, ErrorFallback, RichErrorFallback,
    ErrorContext, ErrorSeverity, create_user_error, handle_error,
};
```

```rust
[component]
pub fn ErrorBoundary(#[prop(into)] children: Children) -> impl IntoView
```
`RichErrorBoundary` uses enhanced `ErrorContext` with severity, suggestions, and optional fallback callback.

---

### Form
**Package:** `leptos_shadcn_form` → `leptos_shadcn_ui::form::*`

```rust
use leptos_shadcn_ui::form::{
    Form, FormField, FormItem, FormLabel, FormControl,
    FormMessage, FormDescription,
    FormData, FormValidation, FormError, FormErrorSeverity,
};
```

```rust
[component]
pub fn Form(
    #[prop(into, optional)] on_submit: Option<Callback<FormData>>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

**FormField** – requires `name: String`, optional `invalid: Signal<bool>`.  
**FormLabel** – requires `for_field: String`.  
**FormMessage** – optional `message: MaybeProp<String>`.  
`FormData` – `.get(field)`, `.get_or_default(field)`.  
`FormValidation` – collections of `FormError` with field, message, rule, severity.

---

### Hover Card
**Package:** `leptos_shadcn_hover_card` → `leptos_shadcn_ui::hover_card::*`

```rust
use leptos_shadcn_ui::hover_card::HoverCard;
```
Same props as DropdownMenu (a simple button).

---

### Input
**Package:** `leptos_shadcn_input` → `leptos_shadcn_ui::input::*`

```rust
use leptos_shadcn_ui::input::Input;
use leptos_shadcn_ui::input::validation::{
    InputValidator, ValidationRule, ValidationResult, validation_builders,
};
```

```rust
[component]
pub fn Input(
    #[prop(into, optional)] value: MaybeProp<String>,
    #[prop(into, optional)] on_change: Option<Callback<String>>,
    #[prop(into, optional)] placeholder: MaybeProp<String>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] input_type: MaybeProp<String>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    // Validation
    #[prop(into, optional)] validator: Option<InputValidator>,
    #[prop(into, optional)] validation_error: MaybeProp<String>,
    #[prop(into, optional)] show_validation: Signal<bool>,
) -> impl IntoView
```

**Validator builders:** `email_validator`, `password_validator`, `username_validator`, `phone_validator`.  
**Rules:** `Required`, `MinLength(usize)`, `MaxLength(usize)`, `Email`, `Pattern(String)`.

---

### Input OTP
**Package:** `leptos_shadcn_input_otp` → `leptos_shadcn_ui::input_otp::*`

```rust
use leptos_shadcn_ui::input_otp::{InputOtp, InputOtpSeparator};
```

```rust
[component]
pub fn InputOtp(
    #[prop(default = 6)] max_length: usize,
    #[prop(optional)] value: MaybeProp<String>,
    #[prop(optional)] on_change: Option<Callback<String>>,
    #[prop(optional)] on_complete: Option<Callback<String>>,
    #[prop(optional)] disabled: MaybeProp<bool>,
    #[prop(optional)] class: MaybeProp<String>,
) -> impl IntoView
```
`InputOtpSeparator` – optional `class`, children.

---

### Label
**Package:** `leptos_shadcn_label` → `leptos_shadcn_ui::label::*`

```rust
use leptos_shadcn_ui::label::Label;
```

```rust
[component]
pub fn Label(
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

---

### Lazy Loading
**Package:** `leptos_shadcn_lazy_loading` → `leptos_shadcn_ui::lazy_loading::*`

```rust
use leptos_shadcn_ui::lazy_loading::{LazyComponentLoader, LazyComponent, BundleAnalyzer};
```
Register components with `LazyComponentLoader`, then use `<LazyComponent name="CompName" />` optionally with fallback and error views.

---

### Menubar
**Package:** `leptos_shadcn_menubar` → `leptos_shadcn_ui::menubar::*`

```rust
use leptos_shadcn_ui::menubar::Menubar;
```
Same style as DropdownMenu (a simple button).

---

### Navigation Menu
**Package:** `leptos_shadcn_navigation_menu` → `leptos_shadcn_ui::navigation_menu::*`

```rust
use leptos_shadcn_ui::navigation_menu::NavigationMenu;
```
Placeholder button.

---

### Pagination
**Package:** `leptos_shadcn_pagination` → `leptos_shadcn_ui::pagination::*`

```rust
use leptos_shadcn_ui::pagination::{
    Pagination, PaginationContent, PaginationItem,
    PaginationLink, PaginationPrevious, PaginationNext, PaginationEllipsis,
};
```

```rust
[component]
pub fn Pagination(
    #[prop(optional)] current_page: MaybeProp<usize>,
    #[prop(default = 1)] total_pages: usize,
    #[prop(optional)] on_page_change: Option<Callback<usize>>,
    #[prop(optional)] show_previous_next: MaybeProp<bool>,
    #[prop(optional)] show_first_last: MaybeProp<bool>,
    #[prop(optional)] class: MaybeProp<String>,
) -> impl IntoView
```
`PaginationLink` – `is_active`, `disabled`, `on_click`.  
`PaginationPrevious`/`Next` – `disabled`, `on_click`, optional children.

---

### Popover
**Package:** `leptos_shadcn_popover` → `leptos_shadcn_ui::popover::*`

```rust
use leptos_shadcn_ui::popover::Popover;
```
Placeholder button.

---

### Progress
**Package:** `leptos_shadcn_progress` → `leptos_shadcn_ui::progress::*`

```rust
use leptos_shadcn_ui::progress::{
    Progress, ProgressRoot, ProgressIndicator, ProgressLabel, ProgressVariant,
};
```

```rust
[component]
pub fn Progress(
    #[prop(into, optional)] value: Signal<f64>,
    #[prop(into, optional)] max: MaybeProp<f64>,   // default 100
    #[prop(into, optional)] variant: MaybeProp<ProgressVariant>,
    #[prop(into, optional)] animated: Signal<bool>,
    #[prop(into, optional)] show_label: Signal<bool>,
    #[prop(into, optional)] size: MaybeProp<String>, // "sm", "lg", "xl" (default "h-3")
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
) -> impl IntoView
```

**Enum:** `ProgressVariant` – `Default`, `Success`, `Warning`, `Destructive`, `Info`

---

### Radio Group
**Package:** `leptos_shadcn_radio_group` → `leptos_shadcn_ui::radio_group::*`

```rust
use leptos_shadcn_ui::radio_group::{RadioGroup, RadioGroupItem};
```

```rust
[component]
pub fn RadioGroup(
    #[prop(into, optional)] value: MaybeProp<String>,
    #[prop(into, optional)] on_value_change: Option<Callback<String>>,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional, into)] children: Option<ChildrenFn>,
) -> impl IntoView
```

```rust
[component]
pub fn RadioGroupItem(
    #[prop(into)] value: String,
    #[prop(into, optional)] disabled: Signal<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(optional, into)] children: Option<ChildrenFn>,
) -> impl IntoView
```

---

### Resizable
**Package:** `leptos_shadcn_resizable` → `leptos_shadcn_ui::resizable::*`

```rust
use leptos_shadcn_ui::resizable::{
    ResizablePanelGroup, ResizablePanel, ResizableHandle,
    ResizeDirection, ResizableState, ResizableConfig,
};
```

```rust
[component]
pub fn ResizablePanelGroup(
    #[prop(into, optional)] direction: MaybeProp<ResizeDirection>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(into, optional)] keyboard_resize: MaybeProp<bool>,
    #[prop(into, optional)] touch_support: MaybeProp<bool>,
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] on_resize: MaybeProp<Callback<Vec<f64>>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

```rust
[component]
pub fn ResizablePanel(
    #[prop(into, optional)] default_size: MaybeProp<f64>,      // default 50
    #[prop(into, optional)] min_size: MaybeProp<f64>,          // default 10
    #[prop(into, optional)] max_size: MaybeProp<f64>,          // default 90
    #[prop(into, optional)] collapsible: MaybeProp<bool>,
    #[prop(into, optional)] collapsed_size: MaybeProp<f64>,    // default 0
    #[prop(into, optional)] collapsed: MaybeProp<bool>,
    #[prop(into, optional)] class: MaybeProp<String>,
    #[prop(into, optional)] id: MaybeProp<String>,
    #[prop(into, optional)] style: Signal<Style>,
    #[prop(into, optional)] aria_label: MaybeProp<String>,
    #[prop(into, optional)] on_resize: MaybeProp<Callback<f64>>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```

**ResizableHandle** – optional `with_handle`, `disabled`, `keyboard_resize`, `touch_support`, `role`, `aria_label`.

---

### Select
*(Not fully shown in dump; typical API: `Select`, `SelectTrigger`, `SelectContent`, `SelectItem` with `value` and `on_value_change`.)*

---

### Separator
*(A simple `Separator` component that renders an `<hr>` or a div; accepts class, id, style.)*

---

### Sheet
*(Similar to Drawer, slides from edges; `Sheet`, `SheetTrigger`, `SheetContent`, etc.)*

---

### Skeleton
*(Placeholder loading shape; accepts class, id, style, children.)*

---

### Slider
*(A range input; `Slider` with value, min, max, step, on_change, class, etc.)*

---

### Switch
*(A toggle switch; `Switch` with checked, on_change, disabled, class, id, style.)*

---

### Table
*(A data table; `Table`, `TableHeader`, `TableBody`, `TableRow`, `TableCell`; often includes `DataTable` with sorting/filtering.)*

---

### Tabs
**Package:** likely `leptos_shadcn_tabs` → `leptos_shadcn_ui::tabs::*`

```rust
use leptos_shadcn_ui::tabs::{Tabs, TabsList, TabsTrigger, TabsContent};
```

```rust
[component]
pub fn Tabs(
    #[prop(into)] value: RwSignal<String>,
    #[prop(into, optional)] on_value_change: Option<Callback<String>>,
    #[prop(into, optional)] class: MaybeProp<String>,
    children: Children,
) -> impl IntoView
```
`TabsTrigger` – value string, class, children.  
`TabsContent` – value string, class, children.

---

### Textarea
*(Similar to Input, but `<textarea>`; props: value, on_change, placeholder, disabled, class, id, style.)*

---

### Toast
**Package:** `leptos_shadcn_toast` → `leptos_shadcn_ui::toast::*`

Uses `ToastProvider` context; provides functions to create toasts with variants and auto‑dismiss.

---

### Toggle
*(A two‑state button; `Toggle` with pressed, on_change, disabled, class, id, style.)*

---

### Tooltip
**Package:** `leptos_shadcn_tooltip` → `leptos_shadcn_ui::tooltip::*`

```rust
use leptos_shadcn_ui::tooltip::{
    TooltipProvider, Tooltip, TooltipTrigger, TooltipContent, TooltipSide,
};
```

```rust
[component]
pub fn Tooltip(
    #[prop(into, optional)] open: RwSignal<bool>,
    #[prop(into, optional)] on_open_change: Option<Callback<bool>>,
    #[prop(into, optional)] side: Signal<TooltipSide>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView
```
`TooltipTrigger` – as button.  
**Enum:** `TooltipSide` – `Top`, `Right`, `Bottom`, `Left`

---

## Global Types & Helpers

- **MaybeProp<T>** – alias for `Signal<Option<T>>`, can be set with `.into()` or `MaybeProp::from(...)`.
- **Signal<Style>** – from `leptos_style::Style`, used for inline styles.
- **RwSignal<T>** – mutable signal; **ReadSignal<T>** – read‑only.
- **Callback<A>** – standard Leptos callback.
- **Children** / **ChildrenFn** – render children.
- **`as_child`** props – accept a callback that receives `ButtonChildProps`, `AccordionTriggerChildProps`, etc., to customise the rendered element.
- **New York variants** – simply append `NewYork` to the component name (e.g., `ButtonNewYork`). They share the same API but may differ visually.
- **Signal‑managed variants** – `SignalManagedButton`, `EnhancedButton`, etc., provide fine‑grained reactive state and performance monitoring; used identically to standard versions.

---

This reference covers every main component from the umbrella crate. For components not fully detailed here, you can safely infer their props follow the same patterns: `class`, `id`, `style`, `children`, and often a `value`/`on_change` pair. Consult the individual `default.rs` files in the repository for the exact signatures if needed.