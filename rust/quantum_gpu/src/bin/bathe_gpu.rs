//! GPU Bathe VQE, CPU reference comparison, and exploratory Hadamard benchmark.
use ash::vk;
use quantum_core::StateVector;
use quantum_cpu::{bathe_hamiltonian, bathe_vqe};
use quantum_gpu::buffer::StorageBuffer;
use quantum_gpu::compute::{CircuitGate, PreparedCircuit};
use quantum_gpu::context::{ComputeDevice, VulkanContext};
use std::{error::Error, time::Instant};

fn gpu_state(
    compute: &ComputeDevice<'_>,
    instance: &ash::Instance,
    physical: vk::PhysicalDevice,
    storage: &StorageBuffer<'_>,
    angles: [f64; 3],
) -> Result<Vec<f32>, Box<dyn Error>> {
    storage.write_f32(&[1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])?;
    let gates = [
        CircuitGate::Ry {
            target: 1,
            theta: angles[0] as f32,
        },
        CircuitGate::Ry {
            target: 0,
            theta: angles[1] as f32,
        },
        CircuitGate::Cnot {
            control: 1,
            target: 0,
        },
        CircuitGate::Ry {
            target: 0,
            theta: angles[2] as f32,
        },
        CircuitGate::Cnot {
            control: 1,
            target: 0,
        },
    ];
    let circuit = PreparedCircuit::new(compute, instance, physical, storage, 4, &gates)?;
    circuit.execute()?;
    storage.read_f32(8)
}

fn compare_state(gpu: &[f32], cpu: &StateVector, tolerance: f64) {
    for (index, value) in cpu.amplitudes.iter().enumerate() {
        assert!(
            (gpu[2 * index] as f64 - value.re).abs() < tolerance,
            "real amplitude {index}"
        );
        assert!(
            (gpu[2 * index + 1] as f64 - value.im).abs() < tolerance,
            "imag amplitude {index}"
        );
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = VulkanContext::new()?;
    let instance = context.instance();
    let physical = unsafe { instance.enumerate_physical_devices()? }
        .into_iter()
        .next()
        .ok_or("no Vulkan device")?;
    let queues = unsafe { instance.get_physical_device_queue_family_properties(physical) };
    let family = queues
        .iter()
        .position(|q| {
            q.queue_count > 0
                && q.queue_flags.contains(vk::QueueFlags::COMPUTE)
                && !q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        })
        .or_else(|| {
            queues
                .iter()
                .position(|q| q.queue_count > 0 && q.queue_flags.contains(vk::QueueFlags::COMPUTE))
        })
        .ok_or("no compute queue")?;
    let compute = ComputeDevice::new(instance, physical, family as u32)?;
    let storage = StorageBuffer::new(instance, physical, compute.device(), 32)?;
    let reference = bathe_vqe::optimize();
    println!("Bathe CPU VQE energy: {:.12}", reference.energy);
    println!("Optimized angles: {:?}", reference.angles);
    for angles in [[0.0, 0.0, 0.0], [0.5, 1.0, -0.25], reference.angles] {
        let gpu = gpu_state(&compute, instance, physical, &storage, angles)?;
        let cpu = bathe_vqe::prepare_state(angles);
        compare_state(&gpu, &cpu, 2e-5);
        let h = bathe_hamiltonian::padded_hamiltonian();
        let energy: f64 = (0..4)
            .flat_map(|i| (0..4).map(move |j| (i, j)))
            .map(|(i, j)| {
                let real_inner = (gpu[2 * i] as f64) * (gpu[2 * j] as f64)
                    + (gpu[2 * i + 1] as f64) * (gpu[2 * j + 1] as f64);
                real_inner * h[i][j]
            })
            .sum();
        let expected = bathe_vqe::energy(angles);
        assert!(
            (energy - expected).abs() < 2e-4,
            "GPU energy {energy}, CPU {expected}"
        );
        println!(
            "angles={angles:?} CPU={expected:.10} GPU={energy:.10} error={:.3e}",
            (energy - expected).abs()
        );
    }
    println!("GPU Bathe VQE comparison verified");
    println!(
        "Prepared Hadamard benchmark: setup is separate; GPU time includes queue submission and wait, not GPU-only kernel time."
    );
    for qubits in [10u32, 15, 20] {
        let count = 1usize << qubits;
        let data = StorageBuffer::new(instance, physical, compute.device(), (count * 8) as u64)?;
        let mut initial = vec![0.0_f32; count * 2];
        initial[0] = 1.0;
        data.write_f32(&initial)?;
        let iterations = if qubits == 20 { 12 } else { 30 };
        let setup_start = Instant::now();
        let circuit = PreparedCircuit::new(
            &compute,
            instance,
            physical,
            &data,
            count,
            &[CircuitGate::Hadamard { target: 0 }],
        )?;
        let setup_us = setup_start.elapsed().as_secs_f64() * 1e6;
        // Warm up without measuring pipeline creation or first submission.
        for _ in 0..5 {
            circuit.execute()?;
        }
        let start = Instant::now();
        for _ in 0..iterations {
            circuit.execute()?;
        }
        let gpu_us = start.elapsed().as_secs_f64() * 1e6 / iterations as f64;
        let mut cpu = StateVector::zero(qubits as usize);
        let start = Instant::now();
        for _ in 0..iterations {
            cpu.hadamard(0);
        }
        let cpu_us = start.elapsed().as_secs_f64() * 1e6 / iterations as f64;
        let read_start = Instant::now();
        let _readback = data.read_f32(count * 2)?;
        let readback_us = read_start.elapsed().as_secs_f64() * 1e6;
        println!(
            "qubits={qubits}, CPU_f64_us={cpu_us:.3}, GPU_f32_submit_wait_us={gpu_us:.3}, GPU_setup_us={setup_us:.3}, GPU_readback_us={readback_us:.3}"
        );
    }
    Ok(())
}
