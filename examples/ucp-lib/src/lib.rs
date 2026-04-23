use quoin_macros::{component, quoin_render};
#[derive(Clone)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u32,
}

pub fn create_initial_people() -> Vec<Person> {
    vec![
        Person {
            id: 1,
            name: "Alice".to_string(),
            age: 30,
        },
        Person {
            id: 2,
            name: "Bob".to_string(),
            age: 25,
        },
    ]
}

#[derive(Clone)]
pub struct TimelineEvent {
    pub id: u32,
    pub label: String,
    pub timestamp: String,
}

#[derive(Clone)]
pub struct CacheEntry {
    pub id: u32,
    pub key: String,
    pub value: String,
    pub hits: u32,
}

pub fn create_timeline_events() -> Vec<TimelineEvent> {
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

pub fn create_cache_entries() -> Vec<CacheEntry> {
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

component! {
    pub DemoApp {
        state {
            count: u32 = 0,
            selected: String = "Option A".to_string(),
            rows: Vec<Person> = create_initial_people(),
        }

        render {
            // Clone signals before the first closure to avoid borrow issues
            let count_display = count.clone();
            let count_btn = count.clone();
            let selected_display = selected.clone();
            let selected_btn_a = selected.clone();
            let selected_btn_b = selected.clone();
            let rows_signal = rows.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { {move || format!("Count: {}", count_display.get())} }
                        button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| count_btn.clone().update(|c| *c += 1)) { "Increment" }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { {move || format!("Selected: {}", selected_display.get())} }
                        button(class: "px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| selected_btn_a.clone().set("Option A".to_string())) { "Option A" }
                        button(class: "px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| selected_btn_b.clone().set("Option B".to_string())) { "Option B" }
                    }
                    div(class: "text-lg font-semibold") { "People:" }
                    div(class: "flex flex-col gap-1") {
                        {move || rows_signal.get().iter().map(|person| {
                            quoin_render! {
                                div(class: "p-2 bg-gray-800 rounded-md") { {format!("{} ({} years old)", person.name, person.age)} }
                            }
                        }).collect::<Vec<_>>()}
                    }
                }
            }
        }
    }
}

component! {
    pub MiniDevtools {
        state {
            active_tab: usize = 0,
            event_count: u32 = 5,
            timeline_events: Vec<TimelineEvent> = create_timeline_events(),
            cache_entries: Vec<CacheEntry> = create_cache_entries(),
            filter_text: String = String::new(),
        }

        render {
            // Clone signals for various closures
            let active_display = active_tab.clone();
            let active_set = active_tab.clone();
            let event_count_display = event_count.clone();
            let event_count_btn = event_count.clone();
            let filter_signal = filter_text.clone();
            let filter_signal_display = filter_text.clone();
            let timeline_signal = timeline_events.clone();
            let cache_signal = cache_entries.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 size-full overflow-hidden") {
                    div(class: "text-xl font-bold text-white") {
                        "Mini Devtools"
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-sm text-gray-400") {
                            {move || format!("Events: {}", event_count_display.get())}
                        }
                    }
                    div(class: "p-2") {
                        input(class: "px-4 py-2 bg-gray-800 text-white rounded-md",
                              placeholder: "Filter events...",
                              value: filter_signal)
                    }
                    div(class: "text-xs text-green-500") {
                        {move || format!("Filter value: {:?}", filter_signal_display.get())}
                    }

                    tabs(active: active_display.get(), on_click: move |i| active_set.clone().set(i)) {
                        tab(index: 0, label: "Timeline")
                        tab(index: 1, label: "Cache")
                        tab(index: 2, label: "Signals")
                    }

                    button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                           primary: true,
                           on_click: move |_| event_count_btn.clone().update(|c| *c += 1)) {
                        "+ Add Event"
                    }

                    /* ---------- reactive tab content ---------- */
                    {move || if active_tab.get() == 0 {
                        /* Timeline tab */
                        let events = timeline_signal.get();
                        let filter = filter_signal_display.get();
                        let filtered: Vec<TimelineEvent> = events.into_iter()
                            .filter(|e| filter.is_empty() || e.label.to_lowercase().contains(&filter.to_lowercase()))
                            .collect();
                        let filtered_count = filtered.len();
                        let total_count = event_count_display.get();

                        leptos::view! {
                            <div class="flex flex-col gap-1 size-full">
                                <div class="text-sm text-gray-400">
                                    {format!("{} events ({} filtered)", total_count, filtered_count)}
                                </div>
                                {filtered.into_iter().map(|event| {
                                    leptos::view! {
                                        <div class="flex gap-4 p-2">
                                            <div class="text-xs text-gray-500">{event.timestamp.clone()}</div>
                                            <div class="text-sm text-white">{event.label.clone()}</div>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }.into_any()
                    } else if active_tab.get() == 1 {
                        /* Cache tab */
                        let entries = cache_signal.get();
                        let is_striped = true;
                        leptos::view! {
                            <Table class="w-full table-striped">
                                <thead>
                                    <tr>
                                        <th class="px-3 py-2 text-gray-400 font-medium">"Key"</th>
                                        <th class="px-3 py-2 text-gray-400 font-medium">"Value"</th>
                                        <th class="px-3 py-2 text-gray-400 font-medium">"Hits"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {entries.iter().map(|row| {
                                        leptos::view! {
                                            <tr>
                                                <td class="px-3 py-2 text-white">{row.key.clone()}</td>
                                                <td class="px-3 py-2 text-white">{row.value.clone()}</td>
                                                <td class="px-3 py-2 text-white">{row.hits.to_string()}</td>
                                            </tr>
                                        }
                                    }).collect::<Vec<_>>()}
                                </tbody>
                            </Table>
                        }.into_any()
                    } else {
                        /* Signals tab */
                        leptos::view! {
                            <div class="flex flex-col gap-2 p-4">
                                <div class="text-sm text-gray-400">"Active signals in current scope:"</div>
                                <div class="p-2 bg-gray-800 rounded-md text-sm text-green-500">"active_tab: usize = 0"</div>
                                <div class="p-2 bg-gray-800 rounded-md text-sm text-green-500">
                                    {move || format!("event_count: u32 = {}", event_count_display.get())}
                                </div>
                                <div class="p-2 bg-gray-800 rounded-md text-sm text-green-500">
                                    {move || format!("filter_text: String = {:?}", filter_signal_display.get())}
                                </div>
                            </div>
                        }.into_any()
                    }}
                }
            }
        }
    }
}
