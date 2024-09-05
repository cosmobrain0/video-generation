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

    let start = Instant::now();
    let circles = vec![
        Circle::new((720.0 / 2.0, 720.0 / 2.0), 300.0, 0xFFC0C0C0),
        Circle::new((100.0, 200.0), 80.0, 0xFFFFFFFF),
    ];
    let rectangles = vec![
        Rectangle::new((240.0, 40.0), (80.0, 150.0), 0xFF0000FF),
        Rectangle::new((30.0, 30.0), (200.0, 50.0), 0xFF7FFF00),
    ];

    let pixel_data = execute_gpu(&gpu_instance, circles, rectangles)
        .await
        .unwrap();
    let end = Instant::now();
    println!(
        "Processing took: {time}",
        time = end.duration_since(start).as_secs_f64()
    );

    let start = Instant::now();
    let image = RgbaImage::from_raw(gpu_instance.width, gpu_instance.height, pixel_data)
        .expect("Failed to create image!");

    image.save("test.bmp").expect("Failed to save image!");
    let end = Instant::now();
    println!(
        "Generating and saving the image took: {time}",
        time = end.duration_since(start).as_secs_f64()
    );
}

async fn execute_gpu(
    gpu_instance: &GpuInstance,
    circles: Vec<Circle>,
    rectangles: Vec<Rectangle>,
) -> Option<Vec<u8>> {
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
    let circle_bind_groups: Vec<_> = circles
        .iter()
        .map(|c| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &circle_bind_group_layout,
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

    let rect_bind_group_layout = rect_compute_pipeline.get_bind_group_layout(0);
    let rect_bind_groups: Vec<_> = rectangles
        .iter()
        .map(|r| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &rect_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: output_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: r.create_buffer(&device, width, height).as_entire_binding(),
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

        let mut draw_shape = |circle_compute_pipeline, circle_bind_group| {
            cpass.set_pipeline(circle_compute_pipeline);
            cpass.set_bind_group(0, circle_bind_group, &[]);
            cpass.dispatch_workgroups(width, height, 1);
        };

        for bind_group in &circle_bind_groups {
            draw_shape(&circle_compute_pipeline, bind_group);
        }
        for bind_group in &rect_bind_groups {
            draw_shape(&rect_compute_pipeline, bind_group);
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
