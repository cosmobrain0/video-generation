use std::{cell::RefCell, ops::Deref, rc::Rc};

#[derive(Clone)]
pub struct Signal<T> {
    inner: Rc<RefCell<T>>,
}
impl<T> Signal<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(initial)),
        }
    }

    pub fn map<O>(&self, f: impl Fn(&T) -> O) -> O {
        (f)(unsafe { &*(self.inner.as_ref().as_ptr()) })
    }

    /// Is this a good idea?
    /// Or should this be `&mut self`, with no `Rc<RefCell<_>>` business?
    pub fn update(&self, f: impl Fn(&mut T)) {
        (f)(unsafe { &mut *(self.inner.as_ref().as_ptr()) })
    }
}
impl<T> Signal<T>
where
    T: Clone,
{
    pub fn get(&self) -> T {
        self.map(|x| x.clone())
    }
}

pub struct SignalRead<T> {
    inner: Signal<T>,
}
impl<T> SignalRead<T> {
    pub fn new(initial: T) -> Self {
        Self {
            inner: Signal::new(initial),
        }
    }

    pub fn map<O>(&self, f: impl Fn(&T) -> O) -> O {
        self.inner.map(f)
    }
}
impl<T> From<Signal<T>> for SignalRead<T> {
    fn from(value: Signal<T>) -> Self {
        Self { inner: value }
    }
}
impl<T> SignalRead<T>
where
    T: Clone,
{
    pub fn get(&self) -> T {
        self.map(|x| x.clone())
    }
}

pub struct DerivedSignal<'a, T> {
    computation: Box<dyn Fn() -> T + 'a>,
}
impl<'a, T> DerivedSignal<'a, T> {
    pub fn new(computation: impl Fn() -> T + 'a) -> Self {
        Self {
            computation: Box::new(computation) as Box<dyn Fn() -> T + 'a>,
        }
    }

    pub fn get(&self) -> T {
        (self.computation)()
    }
}

impl<'a, T, F> From<F> for DerivedSignal<'a, T>
where
    F: Fn() -> T + 'a,
{
    fn from(value: F) -> Self {
        DerivedSignal::new(value)
    }
}

impl<'a, T> From<&'a SignalRead<T>> for DerivedSignal<'a, T>
where
    T: Clone,
{
    fn from(value: &'a SignalRead<T>) -> Self {
        DerivedSignal::new(move || value.map(|x| x.clone()))
    }
}
impl<'a, T> From<&'a Signal<T>> for DerivedSignal<'a, T>
where
    T: Clone,
{
    fn from(value: &'a Signal<T>) -> Self {
        DerivedSignal::new(move || value.map(|x| x.clone()))
    }
}
