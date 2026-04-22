#![feature(prelude_import)]
extern crate std;
#[prelude_import]
use std::prelude::rust_2021::*;
use quoin::Signal;
use quoin_macros::{component, quoin_render};
pub struct Person {
    pub id: u32,
    pub name: String,
    pub age: u32,
}
#[automatically_derived]
impl ::core::clone::Clone for Person {
    #[inline]
    fn clone(&self) -> Person {
        Person {
            id: ::core::clone::Clone::clone(&self.id),
            name: ::core::clone::Clone::clone(&self.name),
            age: ::core::clone::Clone::clone(&self.age),
        }
    }
}
pub fn create_initial_people() -> Vec<Person> {
    <[_]>::into_vec(
        ::alloc::boxed::box_new([
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
        ]),
    )
}
pub struct TimelineEvent {
    pub id: u32,
    pub label: String,
    pub timestamp: String,
}
#[automatically_derived]
impl ::core::clone::Clone for TimelineEvent {
    #[inline]
    fn clone(&self) -> TimelineEvent {
        TimelineEvent {
            id: ::core::clone::Clone::clone(&self.id),
            label: ::core::clone::Clone::clone(&self.label),
            timestamp: ::core::clone::Clone::clone(&self.timestamp),
        }
    }
}
pub struct CacheEntry {
    pub id: u32,
    pub key: String,
    pub value: String,
    pub hits: u32,
}
#[automatically_derived]
impl ::core::clone::Clone for CacheEntry {
    #[inline]
    fn clone(&self) -> CacheEntry {
        CacheEntry {
            id: ::core::clone::Clone::clone(&self.id),
            key: ::core::clone::Clone::clone(&self.key),
            value: ::core::clone::Clone::clone(&self.value),
            hits: ::core::clone::Clone::clone(&self.hits),
        }
    }
}
pub fn create_timeline_events() -> Vec<TimelineEvent> {
    <[_]>::into_vec(
        ::alloc::boxed::box_new([
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
        ]),
    )
}
pub fn create_cache_entries() -> Vec<CacheEntry> {
    <[_]>::into_vec(
        ::alloc::boxed::box_new([
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
        ]),
    )
}
pub struct DemoAppProps {
    _phantom: ::std::marker::PhantomData<()>,
}
#[automatically_derived]
impl ::core::clone::Clone for DemoAppProps {
    #[inline]
    fn clone(&self) -> DemoAppProps {
        DemoAppProps {
            _phantom: ::core::clone::Clone::clone(&self._phantom),
        }
    }
}
impl Default for DemoAppProps {
    fn default() -> Self {
        Self {
            _phantom: ::std::marker::PhantomData,
        }
    }
}
pub struct DemoApp {
    props: DemoAppProps,
    count: quoin::GpuiSignal<u32>,
    selected: quoin::GpuiSignal<String>,
    rows: quoin::GpuiSignal<Vec<Person>>,
    _quoin_inputs: quoin::QuoinInputManager,
    _subs: Vec<gpui::Subscription>,
}
impl DemoApp {
    pub fn new(
        cx: &mut gpui::Context<Self>,
        ctx: quoin::GpuiContext,
        props: DemoAppProps,
    ) -> Self {
        use quoin::ReactiveContext;
        use quoin::Signal;
        let count = ctx.create_signal(0);
        let selected = ctx.create_signal("Option A".to_string());
        let rows = ctx.create_signal(create_initial_people());
        Self {
            props,
            count,
            selected,
            rows,
            _quoin_inputs: quoin::QuoinInputManager::new(),
            _subs: Vec::new(),
        }
    }
}
impl gpui::Render for DemoApp {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::*;
        use quoin::Signal;
        let count = self.count.clone();
        let selected = self.selected.clone();
        let rows = self.rows.clone();
        let count_text = ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Count: {0}", count.get()))
        });
        let selected_text = ::alloc::__export::must_use({
            ::alloc::fmt::format(format_args!("Selected: {0}", selected.get()))
        });
        let people_items = {
            let rows = rows.get();
            rows.iter()
                .map(|person| {
                    let text = ::alloc::__export::must_use({
                        ::alloc::fmt::format(
                            format_args!("{0} ({1} years old)", person.name, person.age),
                        )
                    });
                    ::gpui::div()
                        .p(gpui::px(8.0))
                        .bg(gpui::rgb(0x1f2937))
                        .rounded(gpui::px(6.0))
                        .child(text)
                })
                .collect::<Vec<_>>()
        };
        ::gpui::div()
            .flex()
            .flex_col()
            .gap(gpui::px(16.0))
            .p(gpui::px(16.0))
            .bg(gpui::rgb(0x111827))
            .text_color(gpui::white())
            .h_full()
            .child(
                ::gpui::div()
                    .text_2xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .child("Quoin Render Demo"),
            )
            .child(
                ::gpui::div()
                    .flex()
                    .items_center()
                    .gap(gpui::px(8.0))
                    .child(::gpui::div().text_lg().child(count_text))
                    .child(
                        ::gpui::div()
                            .cursor_pointer()
                            .rounded(::gpui::px(6.0))
                            .px(::gpui::px(8.0))
                            .py(::gpui::px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(::gpui::white())
                            .bg(::gpui::rgb(0x4e4e4e))
                            .px(gpui::px(16.0))
                            .py(gpui::px(8.0))
                            .bg(gpui::rgb(0x2563eb))
                            .text_color(gpui::white())
                            .rounded(gpui::px(6.0))
                            .cursor_pointer()
                            .child("Increment")
                            .on_mouse_down(
                                ::gpui::MouseButton::Left,
                                {
                                    let count = count.clone();
                                    let __handler = ::std::rc::Rc::new(move |_| {
                                        count.clone().update(|c| *c += 1)
                                    });
                                    cx.listener(move |_this, _event, _window, _cx| {
                                        __handler(())
                                    })
                                },
                            ),
                    ),
            )
            .child(
                ::gpui::div()
                    .flex()
                    .items_center()
                    .gap(gpui::px(8.0))
                    .child(::gpui::div().text_lg().child(selected_text))
                    .child(
                        ::gpui::div()
                            .cursor_pointer()
                            .rounded(::gpui::px(6.0))
                            .px(::gpui::px(8.0))
                            .py(::gpui::px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(::gpui::white())
                            .bg(::gpui::rgb(0x4e4e4e))
                            .px(gpui::px(16.0))
                            .py(gpui::px(8.0))
                            .bg(gpui::rgb(0x16a34a))
                            .text_color(gpui::white())
                            .rounded(gpui::px(6.0))
                            .cursor_pointer()
                            .child("Option A")
                            .on_mouse_down(
                                ::gpui::MouseButton::Left,
                                {
                                    let selected = selected.clone();
                                    let __handler = ::std::rc::Rc::new(move |_| {
                                        selected.clone().set("Option A".to_string())
                                    });
                                    cx.listener(move |_this, _event, _window, _cx| {
                                        __handler(())
                                    })
                                },
                            ),
                    )
                    .child(
                        ::gpui::div()
                            .cursor_pointer()
                            .rounded(::gpui::px(6.0))
                            .px(::gpui::px(8.0))
                            .py(::gpui::px(8.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_color(::gpui::white())
                            .bg(::gpui::rgb(0x4e4e4e))
                            .px(gpui::px(16.0))
                            .py(gpui::px(8.0))
                            .bg(gpui::rgb(0x9333ea))
                            .text_color(gpui::white())
                            .rounded(gpui::px(6.0))
                            .cursor_pointer()
                            .child("Option B")
                            .on_mouse_down(
                                ::gpui::MouseButton::Left,
                                {
                                    let selected = selected.clone();
                                    let __handler = ::std::rc::Rc::new(move |_| {
                                        selected.clone().set("Option B".to_string())
                                    });
                                    cx.listener(move |_this, _event, _window, _cx| {
                                        __handler(())
                                    })
                                },
                            ),
                    ),
            )
            .child(
                ::gpui::div()
                    .text_lg()
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child("People:"),
            )
            .child(
                ::gpui::div().flex().flex_col().gap(gpui::px(4.0)).children(people_items),
            )
    }
}
pub struct MiniDevtoolsProps {
    _phantom: ::std::marker::PhantomData<()>,
}
#[automatically_derived]
impl ::core::clone::Clone for MiniDevtoolsProps {
    #[inline]
    fn clone(&self) -> MiniDevtoolsProps {
        MiniDevtoolsProps {
            _phantom: ::core::clone::Clone::clone(&self._phantom),
        }
    }
}
impl Default for MiniDevtoolsProps {
    fn default() -> Self {
        Self {
            _phantom: ::std::marker::PhantomData,
        }
    }
}
pub struct MiniDevtools {
    props: MiniDevtoolsProps,
    active_tab: quoin::GpuiSignal<usize>,
    event_count: quoin::GpuiSignal<u32>,
    timeline_events: quoin::GpuiSignal<Vec<TimelineEvent>>,
    cache_entries: quoin::GpuiSignal<Vec<CacheEntry>>,
    filter_text: quoin::GpuiSignal<String>,
    _quoin_inputs: quoin::QuoinInputManager,
    _subs: Vec<gpui::Subscription>,
}
impl MiniDevtools {
    pub fn new(
        cx: &mut gpui::Context<Self>,
        ctx: quoin::GpuiContext,
        props: MiniDevtoolsProps,
    ) -> Self {
        use quoin::ReactiveContext;
        use quoin::Signal;
        let active_tab = ctx.create_signal(0);
        let event_count = ctx.create_signal(5);
        let timeline_events = ctx.create_signal(create_timeline_events());
        let cache_entries = ctx.create_signal(create_cache_entries());
        let filter_text = ctx.create_signal(String::new());
        Self {
            props,
            active_tab,
            event_count,
            timeline_events,
            cache_entries,
            filter_text,
            _quoin_inputs: quoin::QuoinInputManager::new(),
            _subs: Vec::new(),
        }
    }
}
impl gpui::Render for MiniDevtools {
    fn render(
        &mut self,
        window: &mut gpui::Window,
        cx: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui::*;
        use quoin::Signal;
        let active_tab = self.active_tab.clone();
        let event_count = self.event_count.clone();
        let timeline_events = self.timeline_events.clone();
        let cache_entries = self.cache_entries.clone();
        let filter_text = self.filter_text.clone();
        let active_tab_val = active_tab.get();
        let event_count_val = event_count.get();
        let filter_text_val = filter_text.get();
        let cache_entries_val = cache_entries.get();
        let filtered_events: Vec<TimelineEvent> = timeline_events
            .get()
            .into_iter()
            .filter(|e| {
                filter_text_val.is_empty()
                    || e.label.to_lowercase().contains(&filter_text_val.to_lowercase())
            })
            .collect();
        ::gpui::div()
            .flex()
            .flex_col()
            .gap(gpui::px(16.0))
            .p(gpui::px(16.0))
            .bg(gpui::rgb(0x111827))
            .size_full()
            .overflow_hidden()
            .child(
                ::gpui::div()
                    .text_xl()
                    .font_weight(gpui::FontWeight::BOLD)
                    .text_color(gpui::white())
                    .child("Mini Devtools"),
            )
            .child(
                ::gpui::div()
                    .flex()
                    .items_center()
                    .gap(gpui::px(8.0))
                    .child(
                        ::gpui::div()
                            .text_sm()
                            .text_color(gpui::rgb(0x9ca3af))
                            .child(
                                ::alloc::__export::must_use({
                                    ::alloc::fmt::format(
                                        format_args!("Events: {0}", event_count_val),
                                    )
                                }),
                            ),
                    ),
            )
            .child(
                ::gpui::div()
                    .p(gpui::px(8.0))
                    .child({
                        let __input_id: &str = "__quoin_input_filter_text";
                        if !self._quoin_inputs.contains(__input_id) {
                            let __initial_val: String = self.filter_text.get();
                            let __entity = cx
                                .new::<
                                    quoin::InputState,
                                >(|cx| {
                                    let mut __state = quoin::InputState::new(window, cx);
                                    __state.set_placeholder("Filter events...", window, cx);
                                    __state.set_value(__initial_val, window, cx);
                                    __state
                                });
                            let __sub = cx
                                .observe(
                                    &__entity,
                                    |this, __entity, cx| {
                                        let __new_val: String = __entity
                                            .read(cx)
                                            .value()
                                            .to_string();
                                        this.filter_text.set(__new_val);
                                    },
                                );
                            self._quoin_inputs
                                .insert(__input_id.to_string(), __entity, __sub);
                        } else {
                            let __entity = self._quoin_inputs.get(__input_id).unwrap();
                            let __current: String = __entity
                                .read(cx)
                                .value()
                                .to_string();
                            let __desired: String = self.filter_text.get();
                            if __current != __desired {
                                __entity
                                    .update(
                                        cx,
                                        |__state, cx| {
                                            __state.set_value(__desired, window, cx);
                                        },
                                    );
                            }
                        }
                        let __entity = self
                            ._quoin_inputs
                            .get(__input_id)
                            .unwrap()
                            .clone();
                        ::gpui::div()
                            .px(gpui::px(16.0))
                            .py(gpui::px(8.0))
                            .bg(gpui::rgb(0x1f2937))
                            .text_color(gpui::white())
                            .rounded(gpui::px(6.0))
                            .child(quoin::Input::new(&__entity).appearance(false))
                    }),
            )
            .child(
                ::gpui::div()
                    .text_xs()
                    .text_color(gpui::rgb(0x22c55e))
                    .child(
                        ::alloc::__export::must_use({
                            ::alloc::fmt::format(
                                format_args!("Filter value: {0:?}", filter_text_val),
                            )
                        }),
                    ),
            )
            .child({
                let __active = active_tab_val;
                let __on_click = ::std::rc::Rc::new(move |i| active_tab.clone().set(i));
                let __labels: Vec<(usize, String)> = <[_]>::into_vec(
                    ::alloc::boxed::box_new([
                        (0, "Timeline".to_string()),
                        (1, "Cache".to_string()),
                        (2, "Signals".to_string()),
                    ]),
                );
                let __tab_elements: Vec<::gpui::AnyElement> = __labels
                    .iter()
                    .map(|(idx, label)| {
                        let __is_active = *idx == __active;
                        let mut __el = ::gpui::div()
                            .px(::gpui::px(16.0))
                            .py(::gpui::px(8.0))
                            .cursor_pointer()
                            .child(label.clone());
                        if __is_active {
                            __el = __el.text_color(::gpui::white());
                        } else {
                            __el = __el.text_color(::gpui::rgb(0x9ca3af));
                        }
                        let __idx = *idx;
                        let __tab_on_click = __on_click.clone();
                        __el.on_mouse_down(
                                ::gpui::MouseButton::Left,
                                cx
                                    .listener(move |_this, _event, _window, _cx| {
                                        __tab_on_click(__idx)
                                    }),
                            )
                            .into_any_element()
                    })
                    .collect();
                ::gpui::div().flex().children(__tab_elements)
            })
            .child(
                ::gpui::div()
                    .cursor_pointer()
                    .rounded(::gpui::px(6.0))
                    .px(::gpui::px(8.0))
                    .py(::gpui::px(8.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(::gpui::white())
                    .bg(::gpui::rgb(0x2563eb))
                    .px(gpui::px(16.0))
                    .py(gpui::px(8.0))
                    .bg(gpui::rgb(0x2563eb))
                    .text_color(gpui::white())
                    .rounded(gpui::px(6.0))
                    .cursor_pointer()
                    .child("+ Add Event")
                    .on_mouse_down(
                        ::gpui::MouseButton::Left,
                        {
                            let event_count = event_count.clone();
                            let __handler = ::std::rc::Rc::new(move |_| {
                                event_count.clone().update(|c| *c += 1)
                            });
                            cx.listener(move |_this, _event, _window, _cx| {
                                __handler(())
                            })
                        },
                    ),
            )
            .child({
                if active_tab_val == 0 {
                    ::gpui::div()
                        .children(
                            <[_]>::into_vec(
                                ::alloc::boxed::box_new([
                                    ::gpui::div()
                                        .flex()
                                        .flex_col()
                                        .gap(gpui::px(4.0))
                                        .size_full()
                                        .child(
                                            ::gpui::div()
                                                .text_sm()
                                                .text_color(gpui::rgb(0x9ca3af))
                                                .child(
                                                    ::alloc::__export::must_use({
                                                        ::alloc::fmt::format(
                                                            format_args!(
                                                                "{0} timeline events (showing {1} filtered)",
                                                                event_count_val,
                                                                filtered_events.len(),
                                                            ),
                                                        )
                                                    }),
                                                ),
                                        )
                                        .child({
                                            ::gpui::div()
                                                .children(
                                                    filtered_events
                                                        .into_iter()
                                                        .map(|event| {
                                                            ::gpui::div()
                                                                .children(
                                                                    <[_]>::into_vec(
                                                                        ::alloc::boxed::box_new([
                                                                            ::gpui::div()
                                                                                .flex()
                                                                                .gap(gpui::px(16.0))
                                                                                .p(gpui::px(8.0))
                                                                                .child(
                                                                                    ::gpui::div()
                                                                                        .text_xs()
                                                                                        .text_color(gpui::rgb(0x6b7280))
                                                                                        .child(event.timestamp.clone()),
                                                                                )
                                                                                .child(
                                                                                    ::gpui::div()
                                                                                        .text_sm()
                                                                                        .text_color(gpui::white())
                                                                                        .child(event.label.clone()),
                                                                                ),
                                                                        ]),
                                                                    ),
                                                                )
                                                        })
                                                        .collect::<Vec<_>>(),
                                                )
                                        }),
                                ]),
                            ),
                        )
                } else {
                    {
                        if active_tab_val == 1 {
                            ::gpui::div()
                                .children(
                                    <[_]>::into_vec(
                                        ::alloc::boxed::box_new([
                                            {
                                                let __rows = cache_entries_val;
                                                let __header = ::gpui::div()
                                                    .flex()
                                                    .children(
                                                        <[_]>::into_vec(
                                                            ::alloc::boxed::box_new([
                                                                ::gpui::div()
                                                                    .px(::gpui::px(12.0))
                                                                    .py(::gpui::px(8.0))
                                                                    .text_color(::gpui::rgb(0x6b7280))
                                                                    .font_weight(::gpui::FontWeight::MEDIUM)
                                                                    .child("Key".to_string())
                                                                    .into_any_element(),
                                                                ::gpui::div()
                                                                    .px(::gpui::px(12.0))
                                                                    .py(::gpui::px(8.0))
                                                                    .text_color(::gpui::rgb(0x6b7280))
                                                                    .font_weight(::gpui::FontWeight::MEDIUM)
                                                                    .child("Value".to_string())
                                                                    .into_any_element(),
                                                                ::gpui::div()
                                                                    .px(::gpui::px(12.0))
                                                                    .py(::gpui::px(8.0))
                                                                    .text_color(::gpui::rgb(0x6b7280))
                                                                    .font_weight(::gpui::FontWeight::MEDIUM)
                                                                    .child("Hits".to_string())
                                                                    .into_any_element(),
                                                            ]),
                                                        ),
                                                    );
                                                let __row_elements: Vec<::gpui::AnyElement> = __rows
                                                    .iter()
                                                    .enumerate()
                                                    .map(|(__i, __row)| {
                                                        let mut __row_el = ::gpui::div()
                                                            .flex()
                                                            .children(
                                                                <[_]>::into_vec(
                                                                    ::alloc::boxed::box_new([
                                                                        ::gpui::div()
                                                                            .px(::gpui::px(12.0))
                                                                            .py(::gpui::px(8.0))
                                                                            .text_color(::gpui::rgb(0xffffff))
                                                                            .child((|row: &CacheEntry| row.key.clone())(&__row))
                                                                            .into_any_element(),
                                                                        ::gpui::div()
                                                                            .px(::gpui::px(12.0))
                                                                            .py(::gpui::px(8.0))
                                                                            .text_color(::gpui::rgb(0xffffff))
                                                                            .child((|row: &CacheEntry| row.value.clone())(&__row))
                                                                            .into_any_element(),
                                                                        ::gpui::div()
                                                                            .px(::gpui::px(12.0))
                                                                            .py(::gpui::px(8.0))
                                                                            .text_color(::gpui::rgb(0xffffff))
                                                                            .child((|row: &CacheEntry| row.hits.to_string())(&__row))
                                                                            .into_any_element(),
                                                                    ]),
                                                                ),
                                                            );
                                                        if __i % 2 == 1 {
                                                            __row_el = __row_el.bg(::gpui::rgb(0x1a1a2e));
                                                        }
                                                        __row_el.into_any_element()
                                                    })
                                                    .collect::<Vec<_>>();
                                                ::gpui::div()
                                                    .flex_col()
                                                    .gap_1()
                                                    .size_full()
                                                    .child(__header)
                                                    .children(__row_elements)
                                            },
                                        ]),
                                    ),
                                )
                        } else {
                            ::gpui::div()
                                .children(
                                    <[_]>::into_vec(
                                        ::alloc::boxed::box_new([
                                            ::gpui::div()
                                                .flex()
                                                .flex_col()
                                                .gap(gpui::px(8.0))
                                                .p(gpui::px(16.0))
                                                .child(
                                                    ::gpui::div()
                                                        .text_sm()
                                                        .text_color(gpui::rgb(0x9ca3af))
                                                        .child("Active signals in current scope:"),
                                                )
                                                .child(
                                                    ::gpui::div()
                                                        .p(gpui::px(8.0))
                                                        .bg(gpui::rgb(0x1f2937))
                                                        .rounded(gpui::px(6.0))
                                                        .text_sm()
                                                        .text_color(gpui::rgb(0x22c55e))
                                                        .child("active_tab: usize = 0"),
                                                )
                                                .child(
                                                    ::gpui::div()
                                                        .p(gpui::px(8.0))
                                                        .bg(gpui::rgb(0x1f2937))
                                                        .rounded(gpui::px(6.0))
                                                        .text_sm()
                                                        .text_color(gpui::rgb(0x22c55e))
                                                        .child("event_count: u32 = 5"),
                                                )
                                                .child(
                                                    ::gpui::div()
                                                        .p(gpui::px(8.0))
                                                        .bg(gpui::rgb(0x1f2937))
                                                        .rounded(gpui::px(6.0))
                                                        .text_sm()
                                                        .text_color(gpui::rgb(0x22c55e))
                                                        .child(
                                                            ::alloc::__export::must_use({
                                                                ::alloc::fmt::format(
                                                                    format_args!("filter_text: String = {0:?}", filter_text_val),
                                                                )
                                                            }),
                                                        ),
                                                ),
                                        ]),
                                    ),
                                )
                        }
                    }
                }
            })
    }
}
