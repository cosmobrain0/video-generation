@group(0)
@binding(0)
var<storage, read_write> v_indices_output: array<u32>;

struct Uniforms {
    output_value: u32,
}

@group(0)
@binding(1)
var<uniform> uniforms: Uniforms;

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    v_indices_output[global_id.x] = uniforms.output_value;
}
