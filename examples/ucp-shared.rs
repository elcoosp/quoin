use quoin::Signal;

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

pub fn increment(count: &impl Signal<u32>) {
    count.update(|c| *c += 1);
}

pub fn select_option_a(selected: &impl Signal<String>) {
    selected.set("Option A".to_string());
}

pub fn select_option_b(selected: &impl Signal<String>) {
    selected.set("Option B".to_string());
}
