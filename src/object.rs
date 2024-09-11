use vector2::Vector2;
use video_generator_lib::shapes::Shape;

mod circle_object;
pub use circle_object::*;

pub trait Object {
    fn transform(&self) -> Transform;
    fn set_transform(&self, transform: Transform);
    fn to_shapes(&self) -> Vec<Shape>;
    fn children(&self) -> Vec<&dyn Object>;
    fn children_mut(&mut self) -> Vec<&mut dyn Object>;
    fn add_child(&self, child: Box<dyn Object>);
    fn remove_child(&self, index: usize);
    fn parent(&self) -> Option<&dyn Object>;
    fn global_transform(&self) -> Transform {
        if let Some(parent) = self.parent() {
            parent.global_transform().apply(&self.transform())
        } else {
            self.transform()
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

#[derive(Clone, Debug)]
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
