mod object;

use std::{ops::Sub, rc::Rc};

use colorsys::{Hsl, Rgb};
use object::{CircleObject, Object};
use vector2::Vector2;
use video_generator_lib::{node::*, shapes::*, signal::*};

fn generate_frames(save_frame: &mut dyn FnMut(Vec<Shape>)) {
    horizontal_circle(save_frame);
    final_part(save_frame, 0xFFFFFFFF);
    final_part(save_frame, 0);
}

fn horizontal_circle(save_frame: &mut dyn FnMut(Vec<Shape>)) {
    let time = Rc::new(Signal::new(0.0f32));
    let radius = Rc::new(Signal::new(0.0));

    let mut circle = {
        let (radius_1, radius_2) = (Rc::clone(&radius), Rc::clone(&radius));
        let mut circle = CircleObject::new(move || radius_1.get(), || 0xFFFFFFFF);
        let circle_time = Rc::clone(&time);
        let circle_position_x =
            move || 720.0 / 2.0 + circle_time.get().sin() * (720.0 / 2.0 - radius_2.get());
        circle.set_transform(
            (move || {
                Transform::new(
                    Vector2::new(circle_position_x() as f64, 720.0 / 2.0),
                    0.0,
                    1.0,
                )
            })
            .into(),
        );
        circle
    };

    for i in 0..600 {
        time.update(|t| *t += 0.02);
        if i < 200 && radius.get() <= 50.0 {
            radius.update(|r| *r += 1.0);
        } else if i >= 550 && radius.get() > 0.0 {
            println!("Reducing r from {r}", r = radius.get());
            radius.update(|r| *r = 0.0f32.max(*r - 1.0));
        }

        save_frame(
            [RectangleData::new_shape((0.0, 0.0), (720.0, 720.0), 0)]
                .into_iter()
                .chain(circle.to_shapes_recursive().into_iter())
                .collect(),
        );
    }
}

fn final_part(save_frame: &mut dyn FnMut(Vec<Shape>), colour: u32) {
    let time = Rc::new(Signal::new(0.0f32));
    let fast_time = {
        let time = Rc::clone(&time);
        Rc::new(move || time.get() * 3.0)
    };
    let radius = Rc::new(Signal::new(0.0));

    let mut circle = {
        let (radius_1, radius_2) = (Rc::clone(&radius), Rc::clone(&radius));
        let mut circle = CircleObject::new(move || radius_1.get(), move || colour);
        let circle_time = Rc::clone(&time);
        let circle_position_x =
            move || 720.0 / 2.0 + circle_time.get().sin() * (720.0 / 2.0 - radius_2.get());
        circle.set_transform(
            (move || {
                Transform::new(
                    Vector2::new(circle_position_x() as f64, 720.0 / 2.0),
                    0.0,
                    1.0,
                )
            })
            .into(),
        );
        circle
    };

    let other_circle = {
        let radius = Rc::clone(&radius);
        let fast_time_x = Rc::clone(&fast_time);
        let fast_time_y = Rc::clone(&fast_time);
        let moon_x = move || fast_time_x().sin() * 80.0;
        let moon_y = move || fast_time_y().cos() * 80.0;
        let mut other_circle = CircleObject::new(move || radius.get() / 2.0, || 0xFF0000FF);
        other_circle.set_transform(
            (move || Transform::new(Vector2::new(moon_x() as f64, moon_y() as f64), 0.0, 1.0))
                .into(),
        );
        Rc::new(other_circle)
    };
    circle.set_children(DerivedSignal::new(move || {
        vec![Rc::clone(&other_circle) as Rc<dyn Object>]
    }));

    for i in 0..600 {
        time.update(|t| *t += 0.02);
        if i < 200 && radius.get() <= 50.0 {
            radius.update(|r| *r += 1.0);
        } else if i >= 550 && radius.get() > 0.0 {
            println!("Reducing r from {r}", r = radius.get());
            radius.update(|r| *r = 0.0f32.max(*r - 1.0));
        }

        save_frame(
            [RectangleData::new_shape((0.0, 0.0), (720.0, 720.0), 0)]
                .into_iter()
                .chain(circle.to_shapes_recursive().into_iter())
                .collect(),
        );
    }
}

fn physics_update<'a>(
    mut centre: (f32, f32),
    radius: f32,
    mut velocity: (f32, f32),
) -> ((f32, f32), f32, (f32, f32)) {
    velocity.1 += 0.2;

    centre.0 += velocity.0;
    centre.1 += velocity.1;

    if centre.1 + radius >= 720.0 {
        centre.1 = 720.0 - radius;
        velocity.1 = -velocity.1.abs();
    }

    if centre.0 + radius >= 720.0 {
        centre.0 = 720.0 - radius;
        velocity.0 = -velocity.0.abs();
    }

    if centre.0 - radius <= 0.0 {
        centre.0 = radius;
        velocity.0 = velocity.0.abs();
    }

    (centre, radius, velocity)
}

pub fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (start_frame, end_frame): (usize, usize) =
        (args[0].parse().unwrap(), args[1].parse().unwrap());
    #[cfg(not(target_arch = "wasm32"))]
    {
        pollster::block_on(video_generator_lib::run(
            generate_frames,
            start_frame,
            end_frame,
        ));
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(video_generator_lib::run(
            generate_frames,
            start_frame,
            end_frame,
        ));
    }
}
