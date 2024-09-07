use crate::{Shape, SignalRead};

pub struct Circle<'a> {
    position: (&'a dyn SignalRead<f32>, &'a dyn SignalRead<f32>),
    radius: &'a dyn SignalRead<f32>,
    colour: &'a dyn SignalRead<u32>,
}
impl<'a> Circle<'a> {
    pub fn new() -> Self {
        Self {
            position: (&0.0f32 as &dyn SignalRead<_>, &0.0f32 as &dyn SignalRead<_>),
            radius: &0.0f32 as &dyn SignalRead<_>,
            colour: &0xFF000000u32 as &dyn SignalRead<_>,
        }
    }

    pub fn set_pos_x(&mut self, x: impl SignalRead<f32> + 'a) {
        self.position.0 = &x as &dyn SignalRead<f32>;
    }

    pub fn set_pos_y(&mut self, y: impl SignalRead<f32> + 'a) {
        self.position.1 = &y as &dyn SignalRead<f32>;
    }

    pub fn set_radius(&mut self, radius: impl SignalRead<f32> + 'a) {
        self.radius = &radius as &dyn SignalRead<f32>;
    }

    pub fn set_colour(&mut self, colour: impl SignalRead<u32> + 'a) {
        self.colour = &colour as &dyn SignalRead<u32>;
    }

    pub fn to_shape(&self) -> Shape {
        Shape::Circle(crate::CircleData {
            position: (self.position.0.get(), self.position.1.get()),
            radius: self.radius.get(),
            colour: self.colour.get(),
        })
    }
}
