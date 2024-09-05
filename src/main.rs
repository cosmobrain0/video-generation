mod shapes;

use image::RgbaImage;
use shapes::*;
use std::{borrow::Cow, path::Path, time::Instant};
use wgpu::{util::DeviceExt, ComputePipeline, ShaderModule};

async fn run() {
    let gpu_instance = GpuInstance::new(
        720,
        720,
        Path::new("src/shader.wgsl"),
        Path::new("src/shader-rect.wgsl"),
    )
    .await;

    let format_name = |i| format!("output/test-{i}.bmp");

    let clamp = |x: f32, min, max| x.min(max).max(min);
    let clamp01 = |x| clamp(x, 0.0, 1.0);
    let smoothstep = |x| x * x * (3.0 - 2.0 * x);
    let inverse_lerp = |x, min, max| (x - min) / (max - min);

    println!("Starting...");
    let start = Instant::now();
    let mut frames = Vec::with_capacity(120);
    let mut save_frame = |frame: &Vec<Shape>| frames.push(frame.clone());

    for i in 0..120 {
        let radius = smoothstep(clamp01(inverse_lerp(i as f32, 0.0, 60.0))) * 300.0;
        let height = smoothstep(clamp01(inverse_lerp(i as f32, 30.0, 90.0))) * 60.0;
        let y = smoothstep(clamp01(inverse_lerp(i as f32, 45.0, 105.0))) * 360.0;
        save_frame(&vec![
            Circle::new_shape((720.0 / 2.0, 720.0 / 2.0), radius, 0xFFFFFF),
            Rectangle::new_shape(
                (720.0 / 2.0 - 150.0, y - height / 2.0),
                (300.0, height),
                0xFF0000FF,
            ),
        ]);
    }

    let count = frames.len();
    render_and_save_frames(&gpu_instance, frames, 0, format_name).await;

    let end = Instant::now();
    println!(
        "Time taken for {count} frames is {total_duration}ms - {fps}FPS!",
        total_duration = end.duration_since(start).as_secs_f64(),
        fps = count as f64 / end.duration_since(start).as_secs_f64()
    );
}

async fn render_and_save_frames(
    gpu_instance: &GpuInstance,
    frames: Vec<Vec<Shape>>,
    start_index: usize,
    format_name: impl Fn(usize) -> String,
) {
    for (i, frame) in frames
        .into_iter()
        .enumerate()
        .map(|(i, x)| (i + start_index, x))
    {
        render_and_save_frame(gpu_instance, frame, format_name(i).as_str()).await;
    }
}

async fn render_and_save_frame(gpu_instance: &GpuInstance, shapes: Vec<Shape>, name: &str) {
    let pixel_data = render_frame(&gpu_instance, shapes).await.unwrap();

    let image = RgbaImage::from_raw(gpu_instance.width, gpu_instance.height, pixel_data)
        .expect("Failed to create image!");

    image.save(name).expect("Failed to save image!");
}

async fn render_frame(gpu_instance: &GpuInstance, shapes: Vec<Shape>) -> Option<Vec<u8>> {
    let (width, height, device, circle_compute_pipeline, rect_compute_pipeline) = (
        gpu_instance.width,
        gpu_instance.height,
        &gpu_instance.device,
        &gpu_instance.circle_compute_pipeline,
        &gpu_instance.rect_compute_pipeline,
    );
    let size = (std::mem::size_of::<u8>() as u32 * width * height * 4) as wgpu::BufferAddress;

    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let circle_bind_group_layout = circle_compute_pipeline.get_bind_group_layout(0);
    let rect_bind_group_layout = rect_compute_pipeline.get_bind_group_layout(0);
    let shape_bind_groups: Vec<_> = shapes
        .iter()
        .map(|c| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: match c {
                    Shape::Circle(_) => &circle_bind_group_layout,
                    Shape::Rectangle(_) => &rect_bind_group_layout,
                },
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: c.create_buffer(&device, width, height).as_entire_binding(),
                    },
                ],
            })
        })
        .collect();

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        let mut draw_shape = |compute_pipeline, bind_group| {
            cpass.set_pipeline(compute_pipeline);
            cpass.set_bind_group(0, bind_group, &[]);
            cpass.dispatch_workgroups(width, height, 1);
        };

        for (i, bind_group) in shape_bind_groups.iter().enumerate() {
            draw_shape(
                match &shapes[i] {
                    Shape::Circle(_) => &circle_compute_pipeline,
                    Shape::Rectangle(_) => &rect_compute_pipeline,
                },
                bind_group,
            );
        }
    }
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, size);

    gpu_instance.queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    if let Ok(Ok(())) = receiver.recv_async().await {
        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap();

        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
        pollster::block_on(run());
    }
    #[cfg(target_arch = "wasm32")]
    {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init().expect("could not initialize logger");
        wasm_bindgen_futures::spawn_local(run());
    }
}
