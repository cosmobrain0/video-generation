use std::rc::Rc;

use vector2::Vector2;
use video_generator_lib::{
    shapes::{Shape, Transform},
    signal::DerivedSignal,
};

mod circle_object;
pub use circle_object::*;

pub trait Object<'a> {
    fn transform(&self) -> DerivedSignal<'a, Transform>;
    fn set_transform(&mut self, transform: DerivedSignal<'a, Transform>);

    fn to_shapes(&self) -> Vec<Shape>;

    fn parent(&self) -> DerivedSignal<'a, Transform>;
    fn set_parent(&mut self, parent: DerivedSignal<'a, Transform>);

    fn global_transform(&self) -> DerivedSignal<'a, Transform> {
        let transform = self.transform();
        let parent = self.parent();
        DerivedSignal::new(move || parent.get().apply(&transform.get()))
    }
}
