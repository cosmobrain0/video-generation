mod object;

use std::rc::Rc;

use colorsys::{Hsl, Rgb};
use object::{CircleObject, Object};
use video_generator_lib::{node::*, shapes::*, signal::*};

fn generate_frames(save_frame: &mut dyn FnMut(Vec<Shape>)) {
    let time = Rc::new(Signal::new(0.0f32));
    let fast_time = {
        let time = Rc::clone(&time);
        Rc::new(move || time.get() * 3.0)
    };
    let radius = 50.0;

    let circle_time = Rc::clone(&time);
    let mut circle = CircleObject::new(
        move || 720.0 / 2.0 + circle_time.get().sin() * (720.0 / 2.0 - radius),
        || 720.0 / 2.0,
        move || radius,
        || 0xFFFFFFFF,
    );
    let fast_time_x = Rc::clone(&fast_time);
    let fast_time_y = Rc::clone(&fast_time);
    let mut other_circle = Rc::new(CircleObject::new(
        move || fast_time_x().sin() * 80.0,
        move || fast_time_y().cos() * 80.0,
        move || radius / 2.0,
        || 0xFF0000FF,
    ));
    circle.set_children(DerivedSignal::new(move || {
        vec![Rc::clone(&other_circle) as Rc<dyn Object>]
    }));

    for _ in 0..600 {
        time.update(|t| *t += 0.1);

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
