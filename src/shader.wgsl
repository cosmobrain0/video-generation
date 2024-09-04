@group(0)
@binding(0)
var<storage, read_write> v_indices_output: array<u32>;

struct Uniforms {
    colour: u32,
    width: u32,
}

@group(0)
@binding(1)
var<uniform> uniforms: Uniforms;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x: u32 = global_id.x % uniforms.width;
    let y: u32 = global_id.x / uniforms.width;

    if (x*x + y*y >= uniforms.width*uniforms.width) {
        v_indices_output[global_id.x] = uniforms.colour;
    }
    else {
        v_indices_output[global_id.x] = 0u;
    }
}
