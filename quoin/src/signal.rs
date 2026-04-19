pub trait Signal<T: 'static>: Clone + Copy {
    fn get(&self) -> T;
    fn with<U>(&self, f: impl FnOnce(&T) -> U) -> U;
}
