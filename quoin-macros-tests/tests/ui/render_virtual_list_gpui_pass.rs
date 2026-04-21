use quoin::component;
use quoin::quoin_render;

#[derive(Clone)]
pub struct Event {
    pub label: String,
}

component! {
    pub VirtualListTest {
        state {
            events: Vec<Event> = vec![],
        }
        render {
            let events = events.get();
            quoin_render! {
                div(class: "flex flex-col size-full overflow-y-scroll") {
                    for[event in events] {
                        div(class: "flex gap-3 px-3") {
                            div(class: "text-xs text-gray-500") { event.label.clone() }
                            div(class: "text-sm text-white")    { event.label.clone() }
                        }
                    }
                }
            }
        }
    }
}

fn main() {}
