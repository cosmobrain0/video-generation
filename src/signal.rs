use std::{cell::RefCell, rc::Rc};

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
