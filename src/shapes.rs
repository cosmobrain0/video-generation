use wgpu::{util::DeviceExt as _, Buffer, Device};

pub struct Circle {
    pub position: (f32, f32),
    pub radius: f32,
    pub colour: u32,
}
impl Circle {
    pub fn new(position: (f32, f32), radius: f32, colour: u32) -> Self {
        Self {
            position,
            radius,
            colour,
        }
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
