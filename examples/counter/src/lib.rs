use std::rc::Rc;
use quoin_core::prelude::*;

#[derive(Clone)] // <-- add this
pub struct Counter<S: Signal<u32>> {
    pub count: S,
    pub increment: Rc<dyn Fn()>,
}

pub fn use_counter<C: ReactiveContext>(cx: &C) -> Counter<C::Signal<u32>> {
    let count = cx.create_signal(0u32);
    let increment = {
        let count = count.clone();
        Rc::new(move || {
            count.update(|c| *c += 1);
        })
    };
    Counter { count, increment }
}
