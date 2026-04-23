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
            let c_display = count.clone();
            let c_btn = count.clone();
            let s_display = selected.clone();
            let s_btn_a = selected.clone();
            let s_btn_b = selected.clone();
            let r_rows = rows.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { {move || format!("Count: {}", c_display.get())} }
                        button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| c_btn.clone().update(|c| *c += 1)) { "Increment" }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { {move || format!("Selected: {}", s_display.get())} }
                        button(class: "px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| s_btn_a.clone().set("Option A".to_string())) { "Option A" }
                        button(class: "px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| s_btn_b.clone().set("Option B".to_string())) { "Option B" }
                    }
                    div(class: "text-lg font-semibold") { "People:" }
                    div(class: "flex flex-col gap-1") {
                        {move || r_rows.get().iter().map(|person| {
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
            let a_display = active_tab.clone();
            let a_set = active_tab.clone();
            let e_display_top = event_count.clone();      // for "Events: {…}" text
            let e_btn = event_count.clone();               // for button handler
            let e_display_tab = event_count.clone();       // for tab‑content closures
            let f_filter = filter_text.clone();            // bound to input value
            let f_display_top = filter_text.clone();       // for "Filter value: {…}" text
            let f_display_tab = filter_text.clone();       // for tab‑content closures
            let t_events = timeline_events.clone();
            let c_entries = cache_entries.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 size-full overflow-hidden") {
                    div(class: "text-xl font-bold text-white") {
                        "Mini Devtools"
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-sm text-gray-400") {
                            {move || format!("Events: {}", e_display_top.get())}
                        }
                    }
                    div(class: "p-2") {
                        input(class: "px-4 py-2 bg-gray-800 text-white rounded-md",
                              placeholder: "Filter events...",
                              value: f_filter)
                    }
                    div(class: "text-xs text-green-500") {
                        {move || format!("Filter value: {:?}", f_display_top.get())}
                    }

                    tabs(active: a_display.get(), on_click: move |i| a_set.clone().set(i)) {
                        tab(index: 0, label: "Timeline")
                        tab(index: 1, label: "Cache")
                        tab(index: 2, label: "Signals")
                    }

                    button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                           primary: true,
                           on_click: move |_| e_btn.clone().update(|c| *c += 1)) {
                        "+ Add Event"
                    }

                    /* ----- Reactive tab content ----- */
                    {move || if active_tab.get() == 0 {
                        // Timeline: compute filtered Vec eagerly, then render
                        let events = t_events.get();
                        let filter = f_display_tab.get();
                        let filtered: Vec<TimelineEvent> = events.into_iter()
                            .filter(|e| filter.is_empty() || e.label.to_lowercase().contains(&filter.to_lowercase()))
                            .collect();
                        let filtered_count = filtered.len();
                        let total_count = e_display_tab.get();

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
                        // Cache: use data_table (quoin element)
                        let entries = c_entries.get();
                        quoin_render! {
                            data_table(rows: entries, striped: true) {
                                column(key: "key", label: "Key", render: |row: &CacheEntry| row.key.clone())
                                column(key: "value", label: "Value", render: |row: &CacheEntry| row.value.clone())
                                column(key: "hits", label: "Hits", render: |row: &CacheEntry| row.hits.to_string())
                            }
                        }.into_any()
                    } else {
                        // Signals tab – careful with nested move closures
                        let ec = e_display_tab.clone();
                        let fc = f_display_tab.clone();
                        leptos::view! {
                            <div class="flex flex-col gap-2 p-4">
                                <div class="text-sm text-gray-400">"Active signals in current scope:"</div>
                                <div class="p-2 bg-gray-800 rounded-md text-sm text-green-500">"active_tab: usize = 0"</div>
                                <div class="p-2 bg-gray-800 rounded-md text-sm text-green-500">
                                    {{
                                        let ec2 = ec.clone();
                                        move || format!("event_count: u32 = {}", ec2.get())
                                    }}
                                </div>
                                <div class="p-2 bg-gray-800 rounded-md text-sm text-green-500">
                                    {{
                                        let fc2 = fc.clone();
                                        move || format!("filter_text: String = {:?}", fc2.get())
                                    }}
                                </div>
                            </div>
                        }.into_any()
                    }}
                }
            }
        }
    }
}
