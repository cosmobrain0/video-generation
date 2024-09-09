use std::{borrow::Cow, path::Path};

use wgpu::{util::DeviceExt as _, Adapter, Buffer, ComputePipeline, Device, Queue};

#[derive(Debug, Clone)]
pub struct CircleData {
    pub position: (f32, f32),
    pub radius: f32,
    pub colour: u32,
}
impl CircleData {
    pub fn new(position: (f32, f32), radius: f32, colour: u32) -> Self {
        Self {
            position,
            radius,
            colour,
        }
    }

    pub fn new_shape(position: (f32, f32), radius: f32, colour: u32) -> Shape {
        Shape::Circle(Self {
            position,
            radius,
            colour,
        })
    }

    pub fn create_buffer(&self, device: &Device, width: u32, height: u32) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Uniform Buffer"),
            contents: bytemuck::cast_slice(&[
                bytemuck::cast::<_, u32>(self.position.0),
                bytemuck::cast(self.position.1),
                bytemuck::cast(self.radius),
                width,
                self.colour,
                0,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

#[derive(Debug, Clone)]
pub struct RectangleData {
    pub position: (f32, f32),
    pub size: (f32, f32),
    pub colour: u32,
}
impl RectangleData {
    pub fn new(position: (f32, f32), size: (f32, f32), colour: u32) -> Self {
        Self {
            position,
            size,
            colour,
        }
    }

    pub fn new_shape(position: (f32, f32), size: (f32, f32), colour: u32) -> Shape {
        Shape::Rectangle(Self {
            position,
            size,
            colour,
        })
    }

    pub fn create_buffer(&self, device: &Device, width: u32, height: u32) -> Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Uniform Buffer"),
            contents: bytemuck::cast_slice(&[
                bytemuck::cast::<_, u32>(self.position.0),
                bytemuck::cast(self.position.1),
                bytemuck::cast(self.size.0),
                bytemuck::cast(self.size.1),
                width,
                self.colour,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }
}

pub struct GpuInstance {
    pub width: u32,
    pub height: u32,
    pub instance: wgpu::Instance,
    pub device: Device,
    pub queue: Queue,
    pub circle_compute_pipeline: ComputePipeline,
    pub rect_compute_pipeline: ComputePipeline,
}
impl GpuInstance {
    pub async fn new(width: u32, height: u32, circle_shader: &str, rect_shader: &str) -> Self {
        let instance = wgpu::Instance::default();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
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
        let circle_cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(circle_shader)),
        });
        let rect_cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(rect_shader)),
        });
        let circle_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &circle_cs_module,
                entry_point: "main",
                compilation_options: Default::default(),
                cache: None,
            });

        let rect_compute_pipeline =
            device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                label: None,
                layout: None,
                module: &rect_cs_module,
                entry_point: "main",
                compilation_options: Default::default(),
                cache: None,
            });
        Self {
            width,
            height,
            instance,
            device,
            queue,
            circle_compute_pipeline,
            rect_compute_pipeline,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Shape {
    Circle(CircleData),
    Rectangle(RectangleData),
}
impl Shape {
    pub fn create_buffer(&self, device: &Device, width: u32, height: u32) -> Buffer {
        match self {
            Shape::Circle(x) => x.create_buffer(device, width, height),
            Shape::Rectangle(x) => x.create_buffer(device, width, height),
        }
    }
}
