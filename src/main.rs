use image::RgbaImage;
use std::{borrow::Cow, time::Instant};
use wgpu::util::DeviceExt;

async fn run() {
    let (width, height) = (720, 720);

    let pixel_data = execute_gpu(width, height).await.unwrap();

    let start = Instant::now();
    let image = RgbaImage::from_raw(width, height, pixel_data).expect("Failed to create image!");

    image.save("test.bmp").expect("Failed to save image!");
    let end = Instant::now();
    println!(
        "Generating and saving the image took: {time}",
        time = end.duration_since(start).as_secs_f64()
    );
}

async fn execute_gpu(width: u32, height: u32) -> Option<Vec<u8>> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .unwrap();

    let start = Instant::now();
    let result = execute_gpu_inner(&device, &queue, width, height).await;
    let end = Instant::now();
    println!(
        "Processing took: {time}",
        time = end.duration_since(start).as_secs_f64()
    );
    result
}

async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    width: u32,
    height: u32,
) -> Option<Vec<u8>> {
    // Loads the shader from WGSL
    let circle_cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });
    let rect_cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader-rect.wgsl"))),
    });

    // Gets the size in bytes of the buffer.
    let size = (std::mem::size_of::<u8>() as u32 * width * height * 4) as wgpu::BufferAddress;

    // Instantiates buffer without data.
    // `usage` of buffer specifies how it can be used:
    //   `BufferUsages::MAP_READ` allows it to be read (outside the shader).
    //   `BufferUsages::COPY_DST` allows it to be the destination of the copy.
    let staging_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Instantiates buffer with data (`numbers`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
    let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let uniform_buffer_first_circle =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[
                bytemuck::cast::<_, u32>(720f32 / 2f32),
                bytemuck::cast(720f32 / 2f32),
                bytemuck::cast(300f32),
                width,
                0xC0C0C0FF,
                0,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    let uniform_buffer_second_circle =
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Second Circle Uniform Buffer"),
            contents: bytemuck::cast_slice(&[
                bytemuck::cast::<_, u32>(100f32),
                bytemuck::cast(200f32),
                bytemuck::cast(80f32),
                width,
                0xFFFFFFFF,
                0,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
    let uniform_buffer_first_rect = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("First Rectangle Uniform Buffer"),
        contents: bytemuck::cast_slice(&[
            bytemuck::cast::<_, u32>(100f32),
            bytemuck::cast(200f32),
            bytemuck::cast(80f32),
            bytemuck::cast(30f32),
            width,
            0x00FF00FF,
            0,
        ]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });
    let uniform_buffer_second_rect = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("First Rectangle Uniform Buffer"),
        contents: bytemuck::cast_slice(&[
            bytemuck::cast::<_, u32>(300f32),
            bytemuck::cast(200f32),
            bytemuck::cast(50f32),
            bytemuck::cast(50f32),
            width,
            0xFFFF00FF,
            0,
        ]),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    });

    // A bind group defines how buffers are accessed by shaders.
    // It is to WebGPU what a descriptor set is to Vulkan.
    // `binding` here refers to the `binding` of a buffer in the shader (`layout(set = 0, binding = 0) buffer`).

    // A pipeline specifies the operation of a shader

    // Instantiates the pipeline.
    let circle_compute_pipeline =
        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &circle_cs_module,
            entry_point: "main",
            compilation_options: Default::default(),
            cache: None,
        });

    let rect_compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &rect_cs_module,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    });

    // Instantiates the bind group, once again specifying the binding of buffers.
    let circle_bind_group_layout = circle_compute_pipeline.get_bind_group_layout(0);
    let circle_bind_groups = [
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
                    resource: uniform_buffer_first_circle.as_entire_binding(),
                },
            ],
        }),
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
                    resource: uniform_buffer_second_circle.as_entire_binding(),
                },
            ],
        }),
    ];

    let rect_bind_group_layout = rect_compute_pipeline.get_bind_group_layout(0);
    let rect_bind_groups = [
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
                    resource: uniform_buffer_first_rect.as_entire_binding(),
                },
            ],
        }),
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
                    resource: uniform_buffer_second_rect.as_entire_binding(),
                },
            ],
        }),
    ];

    // A command encoder executes one or many pipelines.
    // It is to WebGPU what a command buffer is to Vulkan.
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        let mut draw_circle = |circle_compute_pipeline, circle_bind_group, debug_name| {
            cpass.set_pipeline(circle_compute_pipeline);
            cpass.set_bind_group(0, circle_bind_group, &[]);
            cpass.insert_debug_marker(debug_name);
            cpass.dispatch_workgroups(width, height, 1);
        };

        draw_circle(
            &circle_compute_pipeline,
            &circle_bind_groups[0],
            "First Circle Render",
        );
        draw_circle(
            &circle_compute_pipeline,
            &circle_bind_groups[1],
            "Second Circle Render",
        );

        let mut draw_rect = |rect_compute_pipeline, rect_bind_group, debug_name| {
            cpass.set_pipeline(rect_compute_pipeline);
            cpass.set_bind_group(0, rect_bind_group, &[]);
            cpass.insert_debug_marker(debug_name);
            cpass.dispatch_workgroups(width, height, 1);
        };

        draw_rect(
            &rect_compute_pipeline,
            &rect_bind_groups[0],
            "First Rectangle Render",
        );
        draw_rect(
            &rect_compute_pipeline,
            &rect_bind_groups[1],
            "Second Rectangle Render",
        );
    }
    // Sets adds copy operation to command encoder.
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(&output_buffer, 0, &staging_buffer, 0, size);

    // Submits command encoder for processing
    queue.submit(Some(encoder.finish()));

    // Note that we're not calling `.await` here.
    let buffer_slice = staging_buffer.slice(..);
    // Sets the buffer up for mapping, sending over the result of the mapping back to us when it is finished.
    let (sender, receiver) = flume::bounded(1);
    buffer_slice.map_async(wgpu::MapMode::Read, move |v| sender.send(v).unwrap());

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    device.poll(wgpu::Maintain::wait()).panic_on_timeout();

    // Awaits until `buffer_future` can be read from
    if let Ok(Ok(())) = receiver.recv_async().await {
        // Gets contents of buffer
        let data = buffer_slice.get_mapped_range();
        // Since contents are got in bytes, this converts these bytes back to u32
        let result = bytemuck::cast_slice(&data).to_vec();

        // With the current interface, we have to make sure all mapped views are
        // dropped before we unmap the buffer.
        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
                                // If you are familiar with C++ these 2 lines can be thought of similarly to:
                                //   delete myPointer;
                                //   myPointer = NULL;
                                // It effectively frees the memory

        // Returns data from buffer
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
