// One invocation transforms one disjoint pair of complex amplitudes.
// Each vec2<f32> holds [real, imaginary].
struct Parameters {
    target_qubit: u32,
    amplitude_count: u32,
}

@group(0) @binding(0)
var<storage, read_write> amplitudes: array<vec2<f32>>;

@group(0) @binding(1)
var<uniform> params: Parameters;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let pair = global_id.x;
    let pair_count = params.amplitude_count / 2u;
    if pair >= pair_count {
        return;
    }

    let stride = 1u << params.target_qubit;
    let block = stride * 2u;
    let low = (pair / stride) * block + (pair % stride);
    let high = low + stride;

    let a = amplitudes[low];
    let b = amplitudes[high];
    let inv_sqrt_2 = inverseSqrt(2.0);

    amplitudes[low] = (a + b) * inv_sqrt_2;
    amplitudes[high] = (a - b) * inv_sqrt_2;
}
