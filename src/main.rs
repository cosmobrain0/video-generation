mod object;

use std::{ops::Sub, rc::Rc};

use colorsys::{Hsl, Rgb};
use object::{CircleObject, Object};
use vector2::Vector2;
use video_generator_lib::{node::*, shapes::*, signal::*};

fn generate_frames(save_frame: &mut dyn FnMut(Vec<Shape>)) {
    final_part(save_frame);
}

fn final_part(save_frame: &mut dyn FnMut(Vec<Shape>)) {
    let smoothstep = |x: f32| x * x * (3.0 - 2.0 * x);
    let smoothstep_clamped = |x: f32| smoothstep(x.min(1.0).max(0.0));

    let time = Signal::new(0.0f32);
    let fast_time = || time.get() * 3.0;
    let normalised_time = || time.get() / 12.0;
    let start_end_ease = |x: f32, a: f32| smoothstep_clamped((0.5 - (x - 0.5).abs()) / a);

    // FIXME: start_and_ease doesn't work
    let radius = || start_end_ease(normalised_time(), 0.1) * 50.0;

    let circle = CircleObject::new(
        || radius(),
        || 0xFFFFFFFF,
        Some(
            (|| {
                Transform::new(
                    Vector2::new(
                        (|| 720.0 / 2.0 + time.get().sin() * (720.0 / 2.0 - radius()))() as f64,
                        720.0 / 2.0,
                    ),
                    0.0,
                    2.0,
                )
            })
            .into(),
        ),
        None,
    );

    let other_circle = CircleObject::new(
        || radius() / 2.0,
        || 0xFF0000FF,
        Some(
            (|| {
                Transform::new(
                    Vector2::new(
                        (|| fast_time().sin() * 80.0)() as f64,
                        (|| fast_time().cos() * 80.0)() as f64,
                    ),
                    0.0,
                    1.0,
                )
            })
            .into(),
        ),
        Some(circle.global_transform()),
    );

    let third_circle = CircleObject::new(
        || radius() / 4.0,
        || 0xFF7FFF00,
        Some((|| Transform::new(Vector2::new(20.0, 0.0), 0.0, 1.0)).into()),
        Some(other_circle.global_transform()),
    );

    for _ in 0..600 {
        time.update(|t| *t += 0.02);

        save_frame(
            [RectangleData::new_shape((0.0, 0.0), (720.0, 720.0), 0)]
                .into_iter()
                .chain(circle.to_shapes())
                .chain(other_circle.to_shapes())
                .chain(third_circle.to_shapes())
                .collect(),
        );
    }
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
