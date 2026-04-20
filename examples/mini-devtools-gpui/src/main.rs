use gpui::*;
use gpui_platform::application;
use quoin::ReactiveContext;
use quoin::Signal;
use quoin_gpui::GpuiContext;
use quoin_macros::quoin_render;
use quoin_ui::SortDirection;

#[derive(Clone)]
struct TimelineEvent {
    id: u32,
    label: String,
    timestamp: String,
}

#[derive(Clone)]
struct CacheEntry {
    id: u32,
    key: String,
    value: String,
    hits: u32,
}

struct MiniDevtools {
    _ctx: GpuiContext,
    active_tab: quoin_gpui::GpuiSignal<usize>,
    event_count: quoin_gpui::GpuiSignal<u32>,
    timeline_events: quoin_gpui::GpuiSignal<Vec<TimelineEvent>>,
    cache_entries: quoin_gpui::GpuiSignal<Vec<CacheEntry>>,
    filter_text: quoin_gpui::GpuiSignal<String>,
    sort_direction: quoin_gpui::GpuiSignal<SortDirection>,
    _quoin_inputs: quoin_ui_gpui::QuoinInputManager,
}

fn create_timeline_events() -> Vec<TimelineEvent> {
    vec![
        TimelineEvent {
            id: 1,
            label: "Component mounted".to_string(),
            timestamp: "0.00ms".to_string(),
        },
        TimelineEvent {
            id: 2,
            label: "Signal updated: count".to_string(),
            timestamp: "1.23ms".to_string(),
        },
        TimelineEvent {
            id: 3,
            label: "Effect fired".to_string(),
            timestamp: "2.45ms".to_string(),
        },
        TimelineEvent {
            id: 4,
            label: "Network request started".to_string(),
            timestamp: "3.67ms".to_string(),
        },
        TimelineEvent {
            id: 5,
            label: "Network response received".to_string(),
            timestamp: "125.4ms".to_string(),
        },
    ]
}

fn create_cache_entries() -> Vec<CacheEntry> {
    vec![
        CacheEntry {
            id: 1,
            key: "user:1".to_string(),
            value: "{name: \"Alice\"}".to_string(),
            hits: 42,
        },
        CacheEntry {
            id: 2,
            key: "user:2".to_string(),
            value: "{name: \"Bob\"}".to_string(),
            hits: 17,
        },
        CacheEntry {
            id: 3,
            key: "config:theme".to_string(),
            value: "\"dark\"".to_string(),
            hits: 99,
        },
    ]
}

impl MiniDevtools {
    fn new(_cx: &mut Context<Self>, ctx: GpuiContext) -> Self {
        Self {
            active_tab: ctx.create_signal(0),
            event_count: ctx.create_signal(5),
            timeline_events: ctx.create_signal(create_timeline_events()),
            cache_entries: ctx.create_signal(create_cache_entries()),
            filter_text: ctx.create_signal(String::new()),
            sort_direction: ctx.create_signal(SortDirection::None),
            _ctx: ctx,
            _quoin_inputs: quoin_ui_gpui::QuoinInputManager::new(),
        }
    }
}

impl Render for MiniDevtools {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // NOTE: `window` (not `_window`) is required for input() support
        let active_tab = self.active_tab.get();
        let event_count = self.event_count.get();
        // NOTE: Do NOT call .get() on filter_text here — pass the signal
        // reference directly to input(value: ...) so the macro can do
        // two-way binding via GpuiSignal::get() / .set().
        let filter_text_val = self.filter_text.get();

        let tab_elements: Vec<AnyElement> = vec![
            "Timeline".to_string(),
            "Cache".to_string(),
            "Signals".to_string(),
        ]
        .iter()
        .enumerate()
        .map(|(i, label)| {
            let tab_index = i;
            let is_active = i == active_tab;
            let mut el = div()
                .px(px(16.0))
                .py(px(8.0))
                .cursor_pointer()
                .child(label.clone());

            if is_active {
                el = el.text_color(white());
            } else {
                el = el.text_color(rgb(0x9ca3af));
            }

            el.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, _, _| {
                    this.active_tab.set(tab_index);
                }),
            )
            .into_any_element()
        })
        .collect();

        let tabs = div().flex().children(tab_elements);

        let tab_content = if active_tab == 0 {
            render_timeline_tab(event_count, &self.timeline_events, filter_text_val.clone())
        } else if active_tab == 1 {
            render_cache_tab(&self.cache_entries)
        } else {
            render_signals_tab(filter_text_val.clone())
        };

        let add_event_btn = quoin_ui_gpui::render_button(
            Some("+ Add Event".to_string()),
            quoin_ui::ButtonVariant {
                primary: true,
                ..Default::default()
            },
        )
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|this, _event, _window, _cx| {
                this.event_count.update(|c| *c += 1);
            }),
        );

        quoin_render! {
            div(class: "flex flex-col gap-4 p-4 bg-gray-900 size-full") {
                div(class: "text-xl font-bold text-white") {
                    "Mini Devtools"
                }
                div(class: "flex items-center gap-2") {
                    div(class: "text-sm text-gray-400") {
                        format!("Events: {}", event_count)
                    }
                }
                div(class: "p-2") {
                    input(class: "px-4 py-2 bg-gray-800 text-white rounded-md", placeholder: "Filter events...", value: self.filter_text)
                }
                div(class: "text-xs text-green-400") {
                    format!("Filter value: {:?}", filter_text_val)
                }
                tabs
                add_event_btn
                tab_content
            }
        }
    }
}

fn render_timeline_tab(
    event_count: u32,
    events: &quoin_gpui::GpuiSignal<Vec<TimelineEvent>>,
    filter_text: String,
) -> AnyElement {
    let rows = events.get();
    let filtered: Vec<_> = rows
        .iter()
        .filter(|e| {
            filter_text.is_empty() || e.label.to_lowercase().contains(&filter_text.to_lowercase())
        })
        .collect();

    let row_elements: Vec<AnyElement> = filtered
        .iter()
        .map(|event| {
            div()
                .flex()
                .gap_4()
                .p_2()
                .child(
                    div()
                        .text_xs()
                        .text_color(rgb(0x6b7280))
                        .child(event.timestamp.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(white())
                        .child(event.label.clone()),
                )
                .into_any_element()
        })
        .collect();

    div()
        .flex_col()
        .gap_1()
        .size_full()
        .child(div().text_sm().text_color(rgb(0x9ca3af)).child(format!(
            "{} timeline events (showing {} filtered)",
            event_count,
            filtered.len()
        )))
        .children(row_elements)
        .into_any_element()
}

fn render_cache_tab(entries: &quoin_gpui::GpuiSignal<Vec<CacheEntry>>) -> AnyElement {
    let header = quoin_ui_gpui::render_table_header(&[
        "Key".to_string(),
        "Value".to_string(),
        "Hits".to_string(),
    ]);

    let rows = entries.get();
    let row_elements: Vec<AnyElement> = rows
        .iter()
        .map(|entry| {
            div()
                .flex()
                .gap_4()
                .p_2()
                .child(
                    div()
                        .w_full()
                        .text_sm()
                        .text_color(rgb(0x3b82f6))
                        .child(entry.key.clone()),
                )
                .child(
                    div()
                        .w_full()
                        .text_sm()
                        .text_color(white())
                        .child(entry.value.clone()),
                )
                .child(
                    div()
                        .text_sm()
                        .text_color(rgb(0x9ca3af))
                        .child(format!("{}", entry.hits)),
                )
                .into_any_element()
        })
        .collect();

    div()
        .flex_col()
        .gap_1()
        .size_full()
        .child(header)
        .children(row_elements)
        .into_any_element()
}

fn render_signals_tab(filter_text: String) -> AnyElement {
    div()
        .flex_col()
        .gap_2()
        .p_4()
        .child(
            div()
                .text_sm()
                .text_color(rgb(0x9ca3af))
                .child("Active signals in current scope:"),
        )
        .child(
            div()
                .p_2()
                .bg(rgb(0x1f2937))
                .rounded(px(6.0))
                .text_sm()
                .text_color(rgb(0x22c55e))
                .child("active_tab: usize = 0"),
        )
        .child(
            div()
                .p_2()
                .bg(rgb(0x1f2937))
                .rounded(px(6.0))
                .text_sm()
                .text_color(rgb(0x22c55e))
                .child("event_count: u32 = 5"),
        )
        .child(
            div()
                .p_2()
                .bg(rgb(0x1f2937))
                .rounded(px(6.0))
                .text_sm()
                .text_color(rgb(0x22c55e))
                .child(format!("filter_text: String = {:?}", filter_text)),
        )
        .into_any_element()
}

fn main() {
    gpui_platform::application()
        .with_assets(gpui_component_assets::Assets)
        .run(|app_cx: &mut App| {
            gpui_component::init(app_cx);

            app_cx
                .open_window(WindowOptions::default(), |window, window_cx| {
                    let mini_devtools = window_cx.new(|cx: &mut Context<MiniDevtools>| {
                        let ctx: GpuiContext = cx.into();
                        ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
                        MiniDevtools::new(cx, ctx)
                    });

                    // Root sets up the per-view theme context required by Input, Button, etc.
                    window_cx.new(|cx| gpui_component::Root::new(mini_devtools, window, cx))
                })
                .unwrap();
            app_cx.activate(true);
        });
}
