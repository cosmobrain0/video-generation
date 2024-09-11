use std::{borrow::Cow, path::Path};

use vector2::Vector2;
use wgpu::{util::DeviceExt as _, Adapter, Buffer, ComputePipeline, Device, Queue};

trait Rotatable: Sized {
    fn get_angle(&self) -> f64;
    fn set_angle(&self, angle: f64) -> Self;
    fn from_angle(angle: f64) -> Self;
    fn rotate(&self, rotation: f64) -> Self {
        self.set_angle(self.get_angle() + rotation)
    }
}
impl Rotatable for Vector2 {
    fn get_angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    fn set_angle(&self, angle: f64) -> Self {
        let length = self.magnitude();
        Vector2::new(angle.cos() * length, angle.sin() * length)
    }

    fn from_angle(angle: f64) -> Self {
        Vector2::new(angle.cos(), angle.sin())
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector2,
    pub rotation: f64,
    pub scale: f64,
}
impl Transform {
    pub fn new(position: Vector2, rotation: f64, scale: f64) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn transform_direction(&self, direction: Vector2) -> Vector2 {
        direction.rotate(self.rotation) * self.scale
    }

    pub fn transform_position(&self, position: Vector2) -> Vector2 {
        position.rotate(self.rotation) * self.scale + self.position
    }

    pub fn apply(&self, other: &Transform) -> Transform {
        Transform::new(
            other.transform_position(self.position),
            self.rotation + other.rotation,
            self.scale * other.scale,
        )
    }
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector2::ZERO,
            rotation: 0.0,
            scale: 1.0,
        }
    }
}

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
        let (x, y, _, _) = self.bounding_box();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Uniform Buffer"),
            contents: bytemuck::cast_slice(&[
                bytemuck::cast(self.radius),
                width,
                x,
                y,
                self.colour,
            ]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn bounding_box(&self) -> (u32, u32, u32, u32) {
        (
            (self.position.0 - self.radius).floor() as u32,
            (self.position.1 - self.radius).floor() as u32,
            (self.radius * 2.0).floor() as u32,
            (self.radius * 2.0).floor() as u32,
        )
    }

    pub fn apply_transform(&self, transform: &Transform) -> Self {
        let position = transform
            .transform_position(Vector2::new(self.position.0 as f64, self.position.1 as f64));
        Self {
            position: (position.x as f32, position.y as f32),
            radius: self.radius * transform.scale as f32,
            colour: self.colour,
        }
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
        let (x, y, _, _) = self.bounding_box();
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Circle Uniform Buffer"),
            contents: bytemuck::cast_slice(&[width, x, y, self.colour]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        })
    }

    pub fn bounding_box(&self) -> (u32, u32, u32, u32) {
        (
            self.position.0.floor() as u32,
            self.position.1.floor() as u32,
            self.size.0.floor() as u32,
            self.size.1.floor() as u32,
        )
    }

    pub fn apply_transform(&self, transform: &Transform) -> Self {
        todo!("Not implemented: RectangleData::apply_transform!")
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

    pub fn bounding_box(&self) -> (u32, u32, u32, u32) {
        match self {
            Shape::Circle(x) => x.bounding_box(),
            Shape::Rectangle(x) => x.bounding_box(),
        }
    }

    pub fn apply_transform(&self, transform: &Transform) -> Shape {
        match self {
            Shape::Circle(x) => Shape::Circle(x.apply_transform(transform)),
            Shape::Rectangle(x) => Shape::Rectangle(x.apply_transform(transform)),
        }
    }
}
