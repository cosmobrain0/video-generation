mod object;

use colorsys::{Hsl, Rgb};
use video_generator_lib::{node::*, shapes::*, signal::*};

fn generate_frames(save_frame: &mut dyn FnMut(Vec<Shape>)) {
    let inverse_lerp = |x, min, max| (x - min) / (max - min);
    let centre = Signal::new((720.0 / 2.0, 720.0 / 2.0));
    let velocity = Signal::new((3.0, 0.0));
    let radius = 50.0f32;

    let circle = Circle::new(
        || centre.map(|c| c.0),
        || centre.map(|c| c.1),
        || radius,
        || {
            centre.map(|c| {
                let hue = inverse_lerp(c.0, 0.0, 720.0) * 360.0;
                let saturation = 100.0;
                let luminance = inverse_lerp(c.1, 350.0, 720.0) * 50.0;
                let colour = Hsl::new(hue as f64, saturation as f64, luminance as f64, Some(1.0));
                let [red, green, blue]: [u8; 3] = Rgb::from(colour).into();
                0xFF000000 + ((red as u32) << 16) + ((green as u32) << 8) + blue as u32
            })
        },
    );
    for _ in 0..600 {
        let (new_centre, _, new_velocity) = physics_update(centre.get(), radius, velocity.get());
        centre.update(|c| *c = new_centre);
        velocity.update(|c| *c = new_velocity);

        save_frame(vec![
            RectangleData::new_shape((0.0, 0.0), (720.0, 720.0), 0),
            circle.to_shape(),
            RectangleData::new_shape((720.0 / 2.0, 720.0 / 2.0), (100.0, 200.0), 0xFFFF0000),
        ]);
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
