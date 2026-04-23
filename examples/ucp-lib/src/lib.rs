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
            let c1 = count.clone();
            let c2 = count.clone();
            let s1 = selected.clone();
            let s2 = selected.clone();
            let s3 = selected.clone();
            let r1 = rows.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 text-white h-full") {
                    div(class: "text-2xl font-bold") { "Quoin Render Demo" }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { {move || format!("Count: {}", c1.get())} }
                        button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| c2.clone().update(|c| *c += 1)) { "Increment" }
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-lg") { {move || format!("Selected: {}", s1.get())} }
                        button(class: "px-4 py-2 bg-green-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| s2.clone().set("Option A".to_string())) { "Option A" }
                        button(class: "px-4 py-2 bg-purple-600 text-white rounded-md cursor-pointer",
                            on_click: move |_| s3.clone().set("Option B".to_string())) { "Option B" }
                    }
                    div(class: "text-lg font-semibold") { "People:" }
                    div(class: "flex flex-col gap-1") {
                        {move || r1.get().iter().map(|person| {
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
            let at1 = active_tab.clone();  // for tab display + condition
            let at2 = active_tab.clone();  // for setter
            let ec1 = event_count.clone();
            let ec2 = event_count.clone();
            let ft1 = filter_text.clone();  // input value
            let ft2 = filter_text.clone();  // filter text display
            let ft3 = filter_text.clone();  // used in for loop expression
            let te1 = timeline_events.clone();
            let ce1 = cache_entries.clone();

            quoin_render! {
                div(class: "flex flex-col gap-4 p-4 bg-gray-900 size-full overflow-hidden") {
                    div(class: "text-xl font-bold text-white") {
                        "Mini Devtools"
                    }
                    div(class: "flex items-center gap-2") {
                        div(class: "text-sm text-gray-400") {
                            {move || format!("Events: {}", ec1.get())}
                        }
                    }
                    div(class: "p-2") {
                        input(class: "px-4 py-2 bg-gray-800 text-white rounded-md",
                              placeholder: "Filter events...",
                              value: ft1)
                    }
                    div(class: "text-xs text-green-500") {
                        {move || format!("Filter value: {:?}", ft2.get())}
                    }

                    tabs(active: at1.get(), on_click: move |i| at2.clone().set(i)) {
                        tab(index: 0, label: "Timeline")
                        tab(index: 1, label: "Cache")
                        tab(index: 2, label: "Signals")
                    }

                    button(class: "px-4 py-2 bg-blue-600 text-white rounded-md cursor-pointer",
                           primary: true,
                           on_click: move |_| ec2.clone().update(|c| *c += 1)) {
                        "+ Add Event"
                    }

                    if[at1.get() == 0] {
                        div(class: "flex flex-col gap-1 size-full") {
                            div(class: "text-sm text-gray-400") {
                                {move || {
                                    let events = te1.get();
                                    let ft = ft2.get();
                                    let filtered_count = events.iter()
                                        .filter(|e| ft.is_empty() || e.label.to_lowercase().contains(&ft.to_lowercase()))
                                        .count();
                                    format!("{} events ({} filtered)", ec1.get(), filtered_count)
                                }}
                            }
                            for[event in {
                                let events = te1.get();
                                let ft = ft3.get();
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
                    } else if[at1.get() == 1] {
                        data_table(rows: ce1.get(), striped: true) {
                            column(key: "key", label: "Key", render: |row: &CacheEntry| row.key.clone())
                            column(key: "value", label: "Value", render: |row: &CacheEntry| row.value.clone())
                            column(key: "hits", label: "Hits", render: |row: &CacheEntry| row.hits.to_string())
                        }
                    } else {
                        div(class: "flex flex-col gap-2 p-4") {
                            div(class: "text-sm text-gray-400") { "Active signals in current scope:" }
                            div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") { "active_tab: usize = 0" }
                            div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") {
                                {move || format!("event_count: u32 = {}", ec1.get())}
                            }
                            div(class: "p-2 bg-gray-800 rounded-md text-sm text-green-500") {
                                {move || format!("filter_text: String = {:?}", ft2.get())}
                            }
                        }
                    }
                }
            }
        }
    }
}
