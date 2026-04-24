use quoin_macros::{component, quoin_render};
#[derive(Clone)]
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u32,
}

pub fn create_initial_people() -> Vec<Person> {
    vec![
        Person { id: 1, name: "Alice".to_string(), age: 30 },
        Person { id: 2, name: "Bob".to_string(), age: 25 },
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
        TimelineEvent { id: 1, label: "Component mounted".to_string(), timestamp: "0.00ms".to_string() },
        TimelineEvent { id: 2, label: "Signal updated: count".to_string(), timestamp: "1.23ms".to_string() },
        TimelineEvent { id: 3, label: "Effect fired".to_string(), timestamp: "2.45ms".to_string() },
        TimelineEvent { id: 4, label: "Network request started".to_string(), timestamp: "3.67ms".to_string() },
        TimelineEvent { id: 5, label: "Network response received".to_string(), timestamp: "125.4ms".to_string() },
    ]
}

pub fn create_cache_entries() -> Vec<CacheEntry> {
    vec![
        CacheEntry { id: 1, key: "user:1".to_string(), value: "{name: \"Alice\"}".to_string(), hits: 42 },
        CacheEntry { id: 2, key: "user:2".to_string(), value: "{name: \"Bob\"}".to_string(), hits: 17 },
        CacheEntry { id: 3, key: "config:theme".to_string(), value: "\"dark\"".to_string(), hits: 99 },
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
            let c_btn     = count.clone();
            let s_display = selected.clone();
            let s_btn_a   = selected.clone();
            let s_btn_b   = selected.clone();
            let r_rows    = rows.clone();

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
            // One clone for each reactive location
            let at_display  = active_tab.clone();
            let at_setter   = active_tab.clone();
            let ec_display  = event_count.clone();
            let ec_btn      = event_count.clone();
            let ft_input    = filter_text.clone();
            let ft_display  = filter_text.clone();
            let ft_timeline = filter_text.clone();  // used inside the for expression
            let te_timeline = timeline_events.clone();
            let ce_cache    = cache_entries.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 size-full overflow-hidden") {
                    div(class: "text-xl font-bold text-white") {
                        "Mini Devtools"
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-sm text-gray-400") {
                            {move || format!("Events: {}", ec_display.get())}
                        }
                    }
                    div(class: "p-2") {
                        input(class: "px-4 py-2 bg-gray-800 text-white rounded-md",
                              placeholder: "Filter events...",
                              value: ft_input)
                    }
                    div(class: "text-xs text-green-500") {
                        {move || format!("Filter value: {:?}", ft_display.get())}
                    }

                    tabs(active: at_display.get(), on_click: move |i| at_setter.clone().set(i)) {
                        tab(index: 0, label: "Timeline")
                        tab(index: 1, label: "Cache")
                        tab(index: 2, label: "Signals")
                    }

                    button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                           primary: true,
                           on_click: move |_| ec_btn.clone().update(|c| *c += 1)) {
                        "+ Add Event"
                    }

                    if[at_display.get() == 0] {
                        div(class: "flex flex-col gap-1 size-full") {
                            div(class: "text-sm text-gray-400") {
                                {move || {
                                    let events = te_timeline.get();
                                    let ft = ft_display.get();
                                    let filtered_count = events.iter()
                                        .filter(|e| ft.is_empty() || e.label.to_lowercase().contains(&ft.to_lowercase()))
                                        .count();
                                    format!("{} events ({} filtered)", ec_display.get(), filtered_count)
                                }}
                            }
                            for[event in {
                                let events = te_timeline.get();
                                let ft = ft_timeline.get();
                                events.into_iter()
                                    .filter(|e| ft.is_empty() || e.label.to_lowercase().contains(&ft.to_lowercase()))
                                    .collect::<Vec<TimelineEvent>>()
                            }] {
                                div(class: "flex gap-4 p-2") {
                                    div(class: "text-xs text-gray-500") { {event.timestamp.clone()} }
                                    div(class: "text-sm text-white") { {event.label.clone()} }
                                }
                            }
                        }
                    } else if[at_display.get() == 1] {
                        div(class: "flex flex-col gap-1 size-full") {
                            for[row in { ce_cache.get() }] {
                                div(class: "flex gap-4 p-2") {
                                    div(class: "text-sm text-white") { {row.key.clone()} }
                                    div(class: "text-sm text-white") { {row.value.clone()} }
                                    div(class: "text-sm text-white") { {row.hits.to_string()} }
                                }
                            }
                        }
                    } else {
                        div(class: "flex flex-col gap-2 p-4") {
                            div(class: "text-sm text-gray-400") { "Active signals in current scope:" }
                            div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") { "active_tab: usize = 0" }
                            div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") {
                                {move || format!("event_count: u32 = {}", ec_display.get())}
                            }
                            div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") {
                                {move || format!("filter_text: String = {:?}", ft_display.get())}
                            }
                        }
                    }
                }
            }
        }
    }
}
