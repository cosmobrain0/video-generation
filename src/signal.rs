pub trait SignalRead<T> {
    fn get(&self) -> T;
}

impl<T> SignalRead<T> for T
where
    T: Clone,
{
    fn get(&self) -> T {
        self.clone()
    }
}

impl<T, A> SignalRead<T> for A
where
    A: Fn() -> T,
{
    fn get(&self) -> T {
        (self)()
    }
}
