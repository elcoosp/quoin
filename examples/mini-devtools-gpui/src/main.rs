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
    fn new(cx: &mut Context<Self>, ctx: GpuiContext) -> Self {
        Self {
            active_tab: ctx.create_signal(0),
            event_count: ctx.create_signal(5),
            timeline_events: ctx.create_signal(create_timeline_events()),
            cache_entries: ctx.create_signal(create_cache_entries()),
            filter_text: ctx.create_signal(String::new()),
            sort_direction: ctx.create_signal(SortDirection::None),
            _ctx: ctx,
        }
    }
}

impl Render for MiniDevtools {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = self.active_tab.get();
        let event_count = self.event_count.get();
        let filter_text = self.filter_text.get();

        let tab_labels = vec![
            "Timeline".to_string(),
            "Cache".to_string(),
            "Signals".to_string(),
        ];

        let tab_elements: Vec<gpui::AnyElement> = tab_labels
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let tab_index = i;
                let is_active = i == active_tab;
                let mut el = gpui::div()
                    .px(gpui::px(16.0))
                    .py(gpui::px(8.0))
                    .cursor_pointer()
                    .child(label.clone());

                if is_active {
                    el = el.text_color(gpui::white());
                } else {
                    el = el.text_color(gpui::rgb(0x9ca3af));
                }

                el.on_mouse_down(
                    gpui::MouseButton::Left,
                    cx.listener(move |this, _, _, _| {
                        this.active_tab.set(tab_index);
                    }),
                )
                .into_any_element()
            })
            .collect();

        let tabs = gpui::div().flex().children(tab_elements);

        let tab_content = if active_tab == 0 {
            render_timeline_tab(event_count, &self.timeline_events)
        } else if active_tab == 1 {
            render_cache_tab(&self.cache_entries)
        } else {
            render_signals_tab()
        };

        let filter = quoin_ui_gpui::render_input(Some("Filter events...".to_string()), filter_text);

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
                tabs
                filter
                add_event_btn
                tab_content
            }
        }
    }
}
fn render_timeline_tab(
    event_count: u32,
    events: &quoin_gpui::GpuiSignal<Vec<TimelineEvent>>,
) -> gpui::AnyElement {
    let rows = events.get();
    let row_elements: Vec<gpui::AnyElement> = rows
        .iter()
        .map(|event| {
            gpui::div()
                .flex()
                .gap_4()
                .p_2()
                .child(
                    gpui::div()
                        .text_xs()
                        .text_color(gpui::rgb(0x6b7280))
                        .child(event.timestamp.clone()),
                )
                .child(
                    gpui::div()
                        .text_sm()
                        .text_color(gpui::white())
                        .child(event.label.clone()),
                )
                .into_any_element()
        })
        .collect();

    gpui::div()
        .flex_col()
        .gap_1()
        .size_full()
        .child(
            gpui::div()
                .text_sm()
                .text_color(gpui::rgb(0x9ca3af))
                .child(format!(
                    "{} timeline events (showing {})",
                    event_count,
                    rows.len()
                )),
        )
        .children(row_elements)
        .into_any_element()
}

fn render_cache_tab(entries: &quoin_gpui::GpuiSignal<Vec<CacheEntry>>) -> gpui::AnyElement {
    let header = quoin_ui_gpui::render_table_header(&[
        "Key".to_string(),
        "Value".to_string(),
        "Hits".to_string(),
    ]);

    let rows = entries.get();
    let row_elements: Vec<gpui::AnyElement> = rows
        .iter()
        .map(|entry| {
            gpui::div()
                .flex()
                .gap_4()
                .p_2()
                .child(
                    gpui::div()
                        .w_full()
                        .text_sm()
                        .text_color(gpui::rgb(0x3b82f6))
                        .child(entry.key.clone()),
                )
                .child(
                    gpui::div()
                        .w_full()
                        .text_sm()
                        .text_color(gpui::white())
                        .child(entry.value.clone()),
                )
                .child(
                    gpui::div()
                        .text_sm()
                        .text_color(gpui::rgb(0x9ca3af))
                        .child(format!("{}", entry.hits)),
                )
                .into_any_element()
        })
        .collect();

    gpui::div()
        .flex_col()
        .gap_1()
        .size_full()
        .child(header)
        .children(row_elements)
        .into_any_element()
}

fn render_signals_tab() -> gpui::AnyElement {
    gpui::div()
        .flex_col()
        .gap_2()
        .p_4()
        .child(
            gpui::div()
                .text_sm()
                .text_color(gpui::rgb(0x9ca3af))
                .child("Active signals in current scope:"),
        )
        .child(
            gpui::div()
                .p_2()
                .bg(gpui::rgb(0x1f2937))
                .rounded(gpui::px(6.0))
                .text_sm()
                .text_color(gpui::rgb(0x22c55e))
                .child("active_tab: usize = 0"),
        )
        .child(
            gpui::div()
                .p_2()
                .bg(gpui::rgb(0x1f2937))
                .rounded(gpui::px(6.0))
                .text_sm()
                .text_color(gpui::rgb(0x22c55e))
                .child("event_count: u32 = 5"),
        )
        .child(
            gpui::div()
                .p_2()
                .bg(gpui::rgb(0x1f2937))
                .rounded(gpui::px(6.0))
                .text_sm()
                .text_color(gpui::rgb(0x22c55e))
                .child("filter_text: String = \"\""),
        )
        .into_any_element()
}

fn main() {
    application().run(|app_cx: &mut App| {
        app_cx
            .open_window(WindowOptions::default(), |window, window_cx| {
                window_cx.new(|cx: &mut Context<MiniDevtools>| {
                    let ctx: GpuiContext = cx.into();
                    ctx.set_view_update_notifier(cx.weak_entity(), window.to_async(cx));
                    MiniDevtools::new(cx, ctx)
                })
            })
            .unwrap();
        app_cx.activate(true);
    });
}
