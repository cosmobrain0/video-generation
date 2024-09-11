use super::{Object, Transform};
use std::rc::Rc;
use vector2::Vector2;
use video_generator_lib::shapes::{CircleData, Shape};
use video_generator_lib::signal::DerivedSignal;

macro_rules! declare_object {
    (pub struct $name:ident { $(pub $property:ident : $type:ty => $setter:ident $(,)?)+ } impl $_:ident { $to_shapes:item} ) => {
        pub struct $name<'a> {
            transform: DerivedSignal<'a, Transform>,
            children: DerivedSignal<'a, Vec<Rc<dyn Object<'a>>>>,
            parent: DerivedSignal<'a, Option<Rc<dyn Object<'a>>>>,
            $(pub $property: DerivedSignal<'a, $type>,)+
        }
        impl<'a> $name<'a> {
            pub fn new($($property: impl Into<DerivedSignal<'a, $type>>,)+) -> Self {
                Self {
                    transform: (|| Default::default()).into(),
                    children: (|| Vec::new()).into(),
                    parent: (|| None).into(),
                    $($property: $property.into(),)+
                }
            }

            $(
                pub fn $setter(&mut self, $property: impl Into<DerivedSignal<'a, $type>>) -> &mut Self {
                    self.$property = $property.into();
                    self
                }
            )+
        }
        impl<'a> Object<'a> for $name<'a> {
            fn transform(&self) -> DerivedSignal<'a, Transform> {
                self.transform.clone()
            }
            fn set_transform(&mut self, transform: DerivedSignal<'a, Transform>) {
                self.transform = transform;
            }
            fn children(&self) -> &DerivedSignal<'a, Vec<Rc<dyn Object<'a>>>> {
                &self.children
            }

            fn set_children(&mut self, children: DerivedSignal<'a, Vec<Rc<dyn Object<'a>>>>) {
                self.children = children;
            }

            fn parent(&self) -> DerivedSignal<'a, Option<Rc<dyn Object<'a>>>> {
                self.parent.clone()
            }

            fn set_parent(&mut self, parent: DerivedSignal<'a, Option<Rc<dyn Object<'a>>>>) {
                self.parent = parent;
            }

            $to_shapes
        }
    }
}

declare_object! {
    pub struct CircleObject {
        pub radius: f32 => set_radius,
        pub colour: u32 => set_colour,
    }
    impl CircleObject {
        fn to_shapes(&self) -> Vec<Shape> {
            let transform = self.transform.get();
            vec![
                Shape::Circle(
                    CircleData::new(
                        (transform.position.x as f32, transform.position.y as f32),
                        self.radius.get() * transform.scale as f32,
                        self.colour.get()
                    )
                )
            ]
        }
    }
}
