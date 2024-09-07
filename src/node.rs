use std::rc::Rc;

use crate::{DerivedSignal, Shape, SignalRead};

pub struct Circle<'a> {
    position: (DerivedSignal<'a, f32>, DerivedSignal<'a, f32>),
    radius: DerivedSignal<'a, f32>,
    colour: DerivedSignal<'a, u32>,
}
impl<'a> Circle<'a> {
    pub fn new() -> Self {
        Self {
            position: (DerivedSignal::new(|| 0.0f32), DerivedSignal::new(|| 0.0f32)),
            radius: DerivedSignal::new(|| 0.0f32),
            colour: DerivedSignal::new(|| 0xFF000000u32),
        }
    }

    pub fn set_pos_x(&mut self, x: DerivedSignal<'a, f32>) {
        self.position.0 = x;
    }

    pub fn set_pos_y(&mut self, y: DerivedSignal<'a, f32>) {
        self.position.1 = y;
    }

    pub fn set_radius(&mut self, radius: DerivedSignal<'a, f32>) {
        self.radius = radius;
    }

    pub fn set_colour(&mut self, colour: DerivedSignal<'a, u32>) {
        self.colour = colour;
    }

    pub fn to_shape(&self) -> Shape {
        Shape::Circle(crate::CircleData {
            position: (self.position.0.get(), self.position.1.get()),
            radius: self.radius.get(),
            colour: self.colour.get(),
        })
    }
}
