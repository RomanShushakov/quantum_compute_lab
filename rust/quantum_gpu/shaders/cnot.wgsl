struct Parameters {
    control_qubit: u32,
    target_qubit: u32,
    amplitude_count: u32,
    unused: u32,
}

@group(0) @binding(0)
var<storage, read_write> amplitudes: array<vec2<f32>>;

@group(0) @binding(1)
var<uniform> params: Parameters;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let pair = id.x;

    if pair >= params.amplitude_count / 2u {
        return;
    }

    let stride = 1u << params.target_qubit;
    let low = (pair / stride) * (2u * stride) + pair % stride;

    if (low & (1u << params.control_qubit)) == 0u {
        return;
    }

    let high = low + stride;
    let a = amplitudes[low];

    amplitudes[low] = amplitudes[high];
    amplitudes[high] = a;
}
