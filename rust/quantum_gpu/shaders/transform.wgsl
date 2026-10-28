@group(0) @binding(0)
var<storage, read_write> values: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;

    if index >= arrayLength(&values) {
        return;
    }

    values[index] = 2.0 * values[index] + 1.0;
}
