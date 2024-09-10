pub mod node;
pub mod shapes;
pub mod signal;

use image::RgbaImage;
use node::Circle;
use shapes::*;
use signal::*;
use std::{path::Path, time::Instant};
use wgpu::Buffer;

pub async fn run(
    generate_frames: impl Fn(&mut dyn FnMut(&Vec<Shape>)),
    start_frame: usize,
    end_frame: usize,
) {
    let gpu_instance = GpuInstance::new(
        720,
        720,
        include_str!("shader.wgsl"),
        include_str!("shader-rect.wgsl"),
    )
    .await;

    let format_name = |i| format!("output/test-{i}.bmp");

    // let clamp = |x: f32, min, max| x.min(max).max(min);
    // let clamp01 = |x| clamp(x, 0.0, 1.0);
    // let smoothstep = |x| x * x * (3.0 - 2.0 * x);

    println!("Starting...");
    let start = Instant::now();
    let mut frames = Vec::with_capacity(120);
    let mut save_frame = |frame: &Vec<Shape>| frames.push(frame.clone());

    generate_frames(&mut save_frame);

    let start_frame = start_frame.min(frames.len() - 1);
    let end_frame = end_frame.min(frames.len() - 1).max(start_frame);
    let count = end_frame - start_frame;
    let generate_frames_end = Instant::now();
    render_and_save_frames(
        &gpu_instance,
        frames
            .into_iter()
            .skip(start_frame)
            .take(end_frame - start_frame),
        0,
        format_name,
    )
    .await;
    let frames_end = Instant::now();
    println!("Saved frames. Exporting video...");
    export_to_video();
    delete_saved_videos(0, count, format_name);

    let end = Instant::now();
    println!(
        "Time taken for {count} frames is {total_duration}ms ({generation_duration} for generating frames, {frame_duration}ms for exporting frames and {video_duration}ms for making a video) - {fps}FPS!",
        total_duration = end.duration_since(start).as_secs_f64(),
        fps = count as f64 / end.duration_since(start).as_secs_f64(),
        generation_duration = generate_frames_end.duration_since(start).as_secs_f64(),
        frame_duration = frames_end.duration_since(generate_frames_end).as_secs_f64(),
        video_duration = end.duration_since(frames_end).as_secs_f64(),
    );
}

fn delete_saved_videos(start_index: usize, count: usize, format_name: impl Fn(usize) -> String) {
    for name in (start_index..count + start_index).map(format_name) {
        std::fs::remove_file(&name)
            .expect(format!("Failed to delete file {name}", name = &name).as_str());
    }
}

fn export_to_video() {
    std::process::Command::new("cmd")
        .args([
            "/C",
            "ffmpeg",
            "-framerate",
            "60",
            "-i",
            "output/test-%d.bmp",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-r",
            "60",
            "output/output.mp4",
            "-y",
        ])
        .stderr(std::process::Stdio::inherit())
        .output()
        .expect("Failed to execute!");
}

async fn render_and_save_frames(
    gpu_instance: &GpuInstance,
    frames: impl Iterator<Item = Vec<Shape>>,
    start_index: usize,
    format_name: impl Fn(usize) -> String,
) {
    let size = (std::mem::size_of::<u8>() as u64
        * gpu_instance.width as u64
        * gpu_instance.height as u64
        * 4) as wgpu::BufferAddress;

    let staging_buffer = gpu_instance.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let output_buffer = gpu_instance.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    for (i, frame) in frames.enumerate().map(|(i, x)| (i + start_index, x)) {
        render_and_save_frame(
            gpu_instance,
            frame,
            format_name(i).as_str(),
            &staging_buffer,
            &output_buffer,
        )
        .await;
    }
}

async fn render_and_save_frame(
    gpu_instance: &GpuInstance,
    shapes: Vec<Shape>,
    name: &str,
    staging_buffer: &Buffer,
    output_buffer: &Buffer,
) {
    let pixel_data = render_frame(&gpu_instance, shapes, staging_buffer, output_buffer)
        .await
        .unwrap();

    let image = RgbaImage::from_raw(gpu_instance.width, gpu_instance.height, pixel_data)
        .expect("Failed to create image!");

    image.save(name).expect("Failed to save image!");
}

async fn render_frame(
    gpu_instance: &GpuInstance,
    shapes: Vec<Shape>,
    staging_buffer: &Buffer,
    output_buffer: &Buffer,
) -> Option<Vec<u8>> {
    let (width, height, device, circle_compute_pipeline, rect_compute_pipeline) = (
        gpu_instance.width,
        gpu_instance.height,
        &gpu_instance.device,
        &gpu_instance.circle_compute_pipeline,
        &gpu_instance.rect_compute_pipeline,
    );

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
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, staging_buffer.size());

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
