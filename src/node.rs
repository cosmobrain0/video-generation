use std::rc::Rc;

use crate::{DerivedSignal, Shape, SignalRead};

pub struct Circle<'a> {
    position: (DerivedSignal<'a, f32>, DerivedSignal<'a, f32>),
    radius: DerivedSignal<'a, f32>,
    colour: DerivedSignal<'a, u32>,
}
impl<'a> Circle<'a> {
    pub fn new(
        pos_x: impl Into<DerivedSignal<'a, f32>>,
        pos_y: impl Into<DerivedSignal<'a, f32>>,
        radius: impl Into<DerivedSignal<'a, f32>>,
        colour: impl Into<DerivedSignal<'a, u32>>,
    ) -> Self {
        Self {
            position: (pos_x.into(), pos_y.into()),
            radius: radius.into(),
            colour: colour.into(),
        }
    }

    pub fn set_pos_x(&mut self, x: impl Into<DerivedSignal<'a, f32>>) -> &mut Self {
        self.position.0 = x.into();
        self
    }

    pub fn set_pos_y(&mut self, y: impl Into<DerivedSignal<'a, f32>>) -> &mut Self {
        self.position.1 = y.into();
        self
    }

    pub fn set_radius(&mut self, radius: impl Into<DerivedSignal<'a, f32>>) -> &mut Self {
        self.radius = radius.into();
        self
    }

    pub fn set_colour(&mut self, colour: impl Into<DerivedSignal<'a, u32>>) -> &mut Self {
        self.colour = colour.into();
        self
    }

    pub fn to_shape(&self) -> Shape {
        Shape::Circle(crate::CircleData {
            position: (self.position.0.get(), self.position.1.get()),
            radius: self.radius.get(),
            colour: self.colour.get(),
        })
    }
}
impl Default for Circle {
    fn default() -> Self {
        Self {
            position: (DerivedSignal::new(|| 0.0f32), DerivedSignal::new(|| 0.0f32)),
            radius: DerivedSignal::new(|| 0.0f32),
            colour: DerivedSignal::new(|| 0xFF000000u32),
        }
    }
}
