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
    fn to_shapes_recursive(&self) -> Vec<Shape> {
        self.children()
            .get()
            .into_iter()
            .map(|x| x.to_shapes_recursive())
            .flatten()
            .map(|x| x.apply_transform(&self.transform().get()))
            .chain(self.to_shapes())
            .collect()
    }

    fn children(&self) -> &DerivedSignal<'a, Vec<Rc<dyn Object<'a>>>>;
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
