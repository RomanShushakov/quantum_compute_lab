//! Repeated multi-gate benchmark. GPU state is reset before every sample.
//! GPU timestamps are reported only when the queue supports them.
use ash::vk;
use quantum_gpu::buffer::StorageBuffer;
use quantum_gpu::compute::{CircuitGate, PreparedCircuit};
use quantum_gpu::context::{ComputeDevice, VulkanContext};
use std::{error::Error, time::Instant};

fn reference(qubits: u32, gates: &[CircuitGate]) -> Vec<f64> {
    let count = 1usize << qubits;
    let mut state = vec![0.0_f64; count * 2];
    state[0] = 1.0;
    for &gate in gates {
        match gate {
            CircuitGate::Hadamard { target } | CircuitGate::Ry { target, .. } => {
                let stride = 1usize << target;
                let (cos, sin, hadamard) = match gate {
                    CircuitGate::Hadamard { .. } => (0.0, 0.0, true),
                    CircuitGate::Ry { theta, .. } => {
                        let half = (theta as f64) * 0.5;
                        (half.cos(), half.sin(), false)
                    }
                    CircuitGate::Cnot { .. } => unreachable!(),
                };
                for pair in 0..count / 2 {
                    let low = (pair / stride) * (2 * stride) + pair % stride;
                    let high = low + stride;
                    for part in 0..2 {
                        let a = state[2 * low + part];
                        let b = state[2 * high + part];
                        let (x, y) = if hadamard {
                            (
                                (a + b) * std::f64::consts::FRAC_1_SQRT_2,
                                (a - b) * std::f64::consts::FRAC_1_SQRT_2,
                            )
                        } else {
                            (cos * a - sin * b, sin * a + cos * b)
                        };
                        state[2 * low + part] = x;
                        state[2 * high + part] = y;
                    }
                }
            }
            CircuitGate::Cnot { control, target } => {
                let stride = 1usize << target;
                for pair in 0..count / 2 {
                    let low = (pair / stride) * (2 * stride) + pair % stride;
                    if low & (1usize << control) != 0 {
                        let high = low + stride;
                        state.swap(2 * low, 2 * high);
                        state.swap(2 * low + 1, 2 * high + 1);
                    }
                }
            }
        }
    }
    state
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.total_cmp(b));
    values[values.len() / 2]
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = VulkanContext::new()?;
    let instance = context.instance();
    let physical = unsafe { instance.enumerate_physical_devices()? }
        .into_iter()
        .next()
        .ok_or("no Vulkan device")?;
    let families = unsafe { instance.get_physical_device_queue_family_properties(physical) };
    let family = families
        .iter()
        .position(|q| {
            q.queue_count > 0
                && q.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && !q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
        .or_else(|| {
            families
                .iter()
                .position(|q| q.queue_count > 0 && q.queue_flags.contains(vk::QueueFlags::COMPUTE))
        })
        .ok_or("no compute queue")?;
    let properties = unsafe { instance.get_physical_device_properties(physical) };
    println!(
        "queue_family={family}, timestamp_valid_bits={}, timestamp_period_ns={}",
        families[family].timestamp_valid_bits, properties.limits.timestamp_period
    );
    let compute = ComputeDevice::new(instance, physical, family as u32)?;
    let gates = [
        CircuitGate::Hadamard { target: 0 },
        CircuitGate::Ry {
            target: 1,
            theta: 0.7,
        },
        CircuitGate::Cnot {
            control: 0,
            target: 1,
        },
        CircuitGate::Hadamard { target: 2 },
        CircuitGate::Ry {
            target: 0,
            theta: -0.4,
        },
        CircuitGate::Cnot {
            control: 2,
            target: 0,
        },
        CircuitGate::Hadamard { target: 1 },
        CircuitGate::Ry {
            target: 2,
            theta: 0.3,
        },
        CircuitGate::Cnot {
            control: 1,
            target: 2,
        },
        CircuitGate::Hadamard { target: 0 },
        CircuitGate::Ry {
            target: 1,
            theta: -0.2,
        },
        CircuitGate::Cnot {
            control: 0,
            target: 1,
        },
    ];
    for qubits in [10u32, 15, 20] {
        let count = 1usize << qubits;
        let storage = StorageBuffer::new(instance, physical, compute.device(), (count * 8) as u64)?;
        let mut initial = vec![0.0_f32; count * 2];
        initial[0] = 1.0;
        let setup_start = Instant::now();
        let circuit = PreparedCircuit::new_timed(
            &compute,
            instance,
            physical,
            &storage,
            count,
            &gates,
            family as u32,
        )?;
        let setup_us = setup_start.elapsed().as_secs_f64() * 1e6;
        let mut submit = Vec::new();
        let mut device = Vec::new();
        let mut read = Vec::new();
        let mut end_to_end = Vec::new();
        let mut cpu = Vec::new();
        let mut max_error = 0.0_f64;
        // Warm-up and measurement each start from |0...0>, so runs are independent.
        for sample in 0..12 {
            let overall = Instant::now();
            storage.write_f32(&initial)?;
            let submit_start = Instant::now();
            circuit.execute()?;
            let submit_us = submit_start.elapsed().as_secs_f64() * 1e6;
            let gpu_us = circuit.gpu_elapsed_us()?;
            let read_start = Instant::now();
            let gpu_state = storage.read_f32(count * 2)?;
            let read_us = read_start.elapsed().as_secs_f64() * 1e6;
            let total_us = overall.elapsed().as_secs_f64() * 1e6;
            let cpu_start = Instant::now();
            let cpu_state = reference(qubits, &gates);
            let cpu_us = cpu_start.elapsed().as_secs_f64() * 1e6;
            for (gpu, cpu_value) in gpu_state.iter().zip(&cpu_state) {
                max_error = max_error.max((*gpu as f64 - *cpu_value).abs());
            }
            if sample >= 3 {
                submit.push(submit_us);
                if let Some(us) = gpu_us {
                    device.push(us);
                }
                read.push(read_us);
                end_to_end.push(total_us);
                cpu.push(cpu_us);
            }
        }
        assert!(max_error < 1e-4, "statevector mismatch: {max_error}");
        let gpu_device = if device.is_empty() {
            "unsupported".to_owned()
        } else {
            format!("{:.3}", median(&mut device))
        };
        println!(
            "qubits={qubits}, gates={}, samples=9, CPU_f64_median_us={:.3}, GPU_f32_submit_wait_median_us={:.3}, GPU_device_median_us={gpu_device}, GPU_readback_median_us={:.3}, GPU_end_to_end_median_us={:.3}, GPU_setup_us={setup_us:.3}, max_amplitude_error={max_error:.3e}",
            gates.len(),
            median(&mut cpu),
            median(&mut submit),
            median(&mut read),
            median(&mut end_to_end)
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reference_hadamard_twice() {
        let result = reference(
            3,
            &[
                CircuitGate::Hadamard { target: 0 },
                CircuitGate::Hadamard { target: 0 },
            ],
        );
        assert!((result[0] - 1.0).abs() < 1e-12);
        assert!(result[1..].iter().all(|x| x.abs() < 1e-12));
    }
}
