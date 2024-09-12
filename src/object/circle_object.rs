use super::{Object, Transform};
use std::rc::Rc;
use vector2::Vector2;
use video_generator_lib::shapes::{CircleData, Shape};
use video_generator_lib::signal::DerivedSignal;

macro_rules! declare_object {
    (pub struct $name:ident { $(pub $property:ident : $type:ty => $setter:ident $(,)?)+ } impl $_:ident { $to_shapes:item} ) => {
        pub struct $name<'a> {
            transform: DerivedSignal<'a, Transform>,
            parent: DerivedSignal<'a, Transform>,
            $(pub $property: DerivedSignal<'a, $type>,)+
        }
        impl<'a> $name<'a> {
            pub fn new($($property: impl Into<DerivedSignal<'a, $type>>,)+ transform: Option<DerivedSignal<'a, Transform>>, parent: Option<DerivedSignal<'a, Transform>>) -> Self {
                Self {
                    transform: transform.unwrap_or((|| Default::default()).into()),
                    parent: parent.unwrap_or((|| Default::default()).into()),
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

            fn parent(&self) -> DerivedSignal<'a, Transform> {
                self.parent.clone()
            }

            fn set_parent(&mut self, parent: DerivedSignal<'a, Transform>) {
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
            let transform = self.global_transform().get();
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
