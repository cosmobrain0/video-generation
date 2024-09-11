use std::rc::Rc;

use vector2::Vector2;
use video_generator_lib::{shapes::Shape, signal::DerivedSignal};

mod circle_object;
pub use circle_object::*;

pub trait Object<'a> {
    fn transform(&self) -> DerivedSignal<'a, Transform>;
    fn set_transform(&mut self, transform: DerivedSignal<'a, Transform>);

    fn to_shapes(&self) -> Vec<Shape>;

    fn children(&self) -> DerivedSignal<'a, Vec<Rc<dyn Object<'a>>>>;
    fn set_children(&mut self, children: DerivedSignal<'a, Vec<Rc<dyn Object<'a>>>>);

    fn parent(&self) -> DerivedSignal<'a, Option<Rc<dyn Object<'a>>>>;
    fn set_parent(&mut self, parent: DerivedSignal<'a, Option<Rc<dyn Object<'a>>>>);

    fn global_transform(&self) -> Transform {
        if let Some(parent) = self.parent().get() {
            parent.global_transform().apply(&self.transform().get())
        } else {
            self.transform().get()
        }
    }
}

trait Rotatable: Sized {
    fn get_angle(&self) -> f64;
    fn set_angle(&self, angle: f64) -> Self;
    fn from_angle(angle: f64) -> Self;
    fn rotate(&self, rotation: f64) -> Self {
        self.set_angle(self.get_angle() + rotation)
    }
}
impl Rotatable for Vector2 {
    fn get_angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    fn set_angle(&self, angle: f64) -> Self {
        let length = self.magnitude();
        Vector2::new(angle.cos() * length, angle.sin() * length)
    }

    fn from_angle(angle: f64) -> Self {
        Vector2::new(angle.cos(), angle.sin())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector2,
    pub rotation: f64,
    pub scale: f64,
}
impl Transform {
    pub fn new(position: Vector2, rotation: f64, scale: f64) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn transform_direction(&self, direction: Vector2) -> Vector2 {
        direction.rotate(self.rotation) * self.scale
    }

    pub fn transform_position(&self, position: Vector2) -> Vector2 {
        position.rotate(self.rotation) * self.scale + self.position
    }

    pub fn apply(&self, other: &Transform) -> Transform {
        Transform::new(
            other.transform_position(self.position),
            self.rotation + other.rotation,
            self.scale * other.scale,
        )
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector2::ZERO,
            rotation: 0.0,
            scale: 1.0,
        }
    }
}
