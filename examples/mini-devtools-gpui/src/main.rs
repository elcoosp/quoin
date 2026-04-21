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
    _sort_direction: quoin_gpui::GpuiSignal<SortDirection>, // Prefixed to suppress warning
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
            _sort_direction: ctx.create_signal(SortDirection::None),
            _ctx: ctx,
            _quoin_inputs: quoin_ui_gpui::QuoinInputManager::new(),
        }
    }
}

impl Render for MiniDevtools {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // 1. Extract all signal reads BEFORE the macro
        let active_tab = self.active_tab.get();
        let event_count = self.event_count.get();
        let filter_text_val = self.filter_text.get();
        let cache_entries = self.cache_entries.get();

        // 2. Clone the signals themselves for use in closures
        let active_tab_signal = self.active_tab.clone();
        let event_count_signal = self.event_count.clone();
        let filter_text_signal = self.filter_text.clone();

        // 3. Compute derived data
        let filtered_events: Vec<TimelineEvent> = self
            .timeline_events
            .get()
            .into_iter()
            .filter(|e| {
                filter_text_val.is_empty()
                    || e.label
                        .to_lowercase()
                        .contains(&filter_text_val.to_lowercase())
            })
            .collect();

        quoin_render! {
            div(class: "flex flex-col gap-4 p-4 bg-gray-900 size-full overflow-y-auto") {
                div(class: "text-xl font-bold text-white") {
                    "Mini Devtools"
                }
                div(class: "flex items-center gap-2") {
                    div(class: "text-sm text-gray-400") {
                        format!("Events: {}", event_count)
                    }
                }
                div(class: "p-2") {
                    input(class: "px-4 py-2 bg-gray-800 text-white rounded-md", placeholder: "Filter events...", value: filter_text_signal)
                }
                div(class: "text-xs text-green-500") {
                    format!("Filter value: {:?}", filter_text_val)
                }

                tabs(active: active_tab, on_click: (|i| active_tab_signal.set(i))) {
                    tab(index: 0, label: "Timeline")
                    tab(index: 1, label: "Cache")
                    tab(index: 2, label: "Signals")
                }

                button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer", primary: true, on_click: (|_| event_count_signal.update(|c| *c += 1))) {
                    "+ Add Event"
                }
                if[active_tab == 0] {
                    div(class: "flex flex-col gap-1 size-full") {
                        div(class: "text-sm text-gray-400") {
                            format!("{} timeline events (showing {} filtered)", event_count, filtered_events.len())
                        }
                        for[event in filtered_events] {
                            div(class: "flex gap-4 p-2") {
                                div(class: "text-xs text-gray-500") { event.timestamp.clone() }
                                div(class: "text-sm text-white") { event.label.clone() }
                            }
                        }
                    }
                } else if[active_tab == 1] {
                    data_table(rows: cache_entries, striped: true) {
                        column(key: "key", label: "Key", render: (|row: &CacheEntry| row.key.clone()))
                        column(key: "value", label: "Value", render: (|row: &CacheEntry| row.value.clone()))
                        column(key: "hits", label: "Hits", render: (|row: &CacheEntry| row.hits.to_string()))
                    }
                } else {
                    div(class: "flex flex-col gap-2 p-4") {
                        div(class: "text-sm text-gray-400") { "Active signals in current scope:" }
                        div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") { "active_tab: usize = 0" }
                        div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") { "event_count: u32 = 5" }
                        div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") { format!("filter_text: String = {:?}", filter_text_val) }
                    }
                }
            }
        }
    }
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

                    window_cx.new(|cx| gpui_component::Root::new(mini_devtools, window, cx))
                })
                .unwrap();
            app_cx.activate(true);
        });
}
