@group(0)
@binding(0)
var<storage, read_write> v_indices_output: array<u32>;

struct Uniforms {
    position: vec2<f32>,
    radius: f32,
    width: u32,
    offset_x: u32,
    offset_y: u32,
    colour: u32,
}

@group(0)
@binding(1)
var<uniform> uniforms: Uniforms;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let global_id_offset: vec2<u32> = vec2<u32>(global_id.x+uniforms.offset_x, global_id.y+uniforms.offset_y);
    let id: u32 = global_id_offset.y*uniforms.width + global_id_offset.x;
    let x: f32 = f32(global_id_offset.x);
    let y: f32 = f32(global_id_offset.y);

    let offset: vec2<f32> = vec2<f32>(x, y) - uniforms.position;

    if (offset.x*offset.x + offset.y*offset.y <= uniforms.radius*uniforms.radius) {
        v_indices_output[id] = uniforms.colour;
    }
}
