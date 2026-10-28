use ash::vk;
use quantum_gpu::buffer::StorageBuffer;
use quantum_gpu::compute::{run_hadamard, run_transform};
use quantum_gpu::context::{ComputeDevice, VulkanContext};
use std::error::Error;
use std::ffi::CStr;

fn verify_compute_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
) -> Result<(), Box<dyn Error>> {
    let compute = ComputeDevice::new(instance, physical_device, queue_family_index)?;
    println!("  Logical device created");
    println!("  Compute queue handle: {:?}", compute.queue());

    let input = [1.0_f32, 2.0, 3.0, 4.0];
    let storage = StorageBuffer::new(
        instance,
        physical_device,
        compute.device(),
        std::mem::size_of_val(&input) as vk::DeviceSize,
    )?;
    storage.write_f32(&input)?;
    let output = storage.read_f32(input.len())?;

    println!(
        "  Storage buffer memory type: {}",
        storage.memory_type_index()
    );
    println!("  Storage buffer input: {input:?}");
    println!("  Storage buffer readback: {output:?}");
    assert_eq!(output, input);
    println!("  Storage buffer allocation and mapping verified");

    compute.submit_empty()?;
    println!("  Empty compute command buffer submitted successfully");

    storage.write_f32(&input)?;
    run_transform(&compute, &storage, input.len())?;
    let gpu_output = storage.read_f32(input.len())?;
    let expected = [3.0_f32, 5.0, 7.0, 9.0];
    println!("  GPU transform output: {gpu_output:?}");
    assert_eq!(gpu_output, expected);
    println!("  GPU compute dispatch verified");

    // Two-qubit |00> state, interleaved real/imaginary f32 components.
    let initial = [1.0_f32, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
    let state = StorageBuffer::new(
        instance,
        physical_device,
        compute.device(),
        std::mem::size_of_val(&initial) as vk::DeviceSize,
    )?;
    state.write_f32(&initial)?;
    run_hadamard(&compute, instance, physical_device, &state, 4, 0)?;
    let result = state.read_f32(initial.len())?;
    let inv_sqrt_2 = std::f32::consts::FRAC_1_SQRT_2;
    let expected = [inv_sqrt_2, 0.0, inv_sqrt_2, 0.0, 0.0, 0.0, 0.0, 0.0];
    assert_approx_eq(&result, &expected, 1e-5);
    println!("  GPU Hadamard target 0: {result:?}");

    run_hadamard(&compute, instance, physical_device, &state, 4, 0)?;
    let restored = state.read_f32(initial.len())?;
    assert_approx_eq(&restored, &initial, 1e-5);
    println!("  GPU Hadamard twice restores the input");

    state.write_f32(&initial)?;
    run_hadamard(&compute, instance, physical_device, &state, 4, 1)?;
    let result = state.read_f32(initial.len())?;
    let expected = [inv_sqrt_2, 0.0, 0.0, 0.0, inv_sqrt_2, 0.0, 0.0, 0.0];
    assert_approx_eq(&result, &expected, 1e-5);
    println!("  GPU Hadamard target 1: {result:?}");

    // Complex input verifies both real and imaginary components.
    let complex_input = [0.5_f32, 0.25, 0.25, -0.5];
    let complex_state = StorageBuffer::new(
        instance,
        physical_device,
        compute.device(),
        std::mem::size_of_val(&complex_input) as vk::DeviceSize,
    )?;
    complex_state.write_f32(&complex_input)?;
    run_hadamard(&compute, instance, physical_device, &complex_state, 2, 0)?;
    let complex_output = complex_state.read_f32(complex_input.len())?;
    let complex_expected = [
        0.75 * inv_sqrt_2,
        -0.25 * inv_sqrt_2,
        0.25 * inv_sqrt_2,
        0.75 * inv_sqrt_2,
    ];
    assert_approx_eq(&complex_output, &complex_expected, 1e-5);
    println!("  GPU Hadamard complex amplitudes verified");
    Ok(())
}

fn assert_approx_eq(actual: &[f32], expected: &[f32], tolerance: f32) {
    assert_eq!(actual.len(), expected.len());
    for (index, (&a, &b)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - b).abs() <= tolerance,
            "amplitude component {index}: expected {b}, got {a}"
        );
    }
}

fn inspect_memory(instance: &ash::Instance, physical_device: vk::PhysicalDevice) {
    let memory_properties =
        unsafe { instance.get_physical_device_memory_properties(physical_device) };

    println!("  Vulkan memory types:");

    for index in 0..memory_properties.memory_type_count as usize {
        let memory_type = memory_properties.memory_types[index];

        println!(
            "    Type {index}: heap={}, flags={:?}",
            memory_type.heap_index, memory_type.property_flags
        );
    }

    println!("  Vulkan memory heaps:");

    for index in 0..memory_properties.memory_heap_count as usize {
        let heap = memory_properties.memory_heaps[index];

        println!(
            "    Heap {index}: size={} MiB, flags={:?}",
            heap.size / (1024 * 1024),
            heap.flags
        );
    }
}

fn inspect_devices(instance: &ash::Instance) -> Result<(), Box<dyn Error>> {
    let devices = unsafe { instance.enumerate_physical_devices()? };

    if devices.is_empty() {
        return Err("no Vulkan physical devices found".into());
    }

    for (device_index, device) in devices.iter().enumerate() {
        let properties = unsafe { instance.get_physical_device_properties(*device) };
        let features = unsafe { instance.get_physical_device_features(*device) };
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(*device) };

        let name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }.to_string_lossy();

        println!("Device {device_index}: {name}");
        println!(
            "  Vulkan API: {}.{}.{}",
            vk::api_version_major(properties.api_version),
            vk::api_version_minor(properties.api_version),
            vk::api_version_patch(properties.api_version)
        );
        println!("  shaderFloat64: {}", features.shader_float64 == vk::TRUE);

        inspect_memory(instance, *device);

        for (family_index, family) in queue_families.iter().enumerate() {
            let compute = family.queue_flags.contains(vk::QueueFlags::COMPUTE);
            let graphics = family.queue_flags.contains(vk::QueueFlags::GRAPHICS);

            println!(
                "  Queue family {family_index}: count={}, compute={compute}, graphics={graphics}",
                family.queue_count
            );
        }

        let selected = queue_families
            .iter()
            .enumerate()
            .find(|(_, family)| {
                family.queue_count > 0
                    && family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                    && !family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            })
            .or_else(|| {
                queue_families.iter().enumerate().find(|(_, family)| {
                    family.queue_count > 0 && family.queue_flags.contains(vk::QueueFlags::COMPUTE)
                })
            });

        if let Some((family_index, _)) = selected {
            println!("  Preferred compute queue family: {family_index}");

            if features.shader_float64 == vk::TRUE {
                verify_compute_device(instance, *device, family_index as u32)?;
            } else {
                println!("  Skipping f64 logical-device creation: shaderFloat64 unavailable");
            }
        } else {
            println!("  No compute-capable queue family found");
        }

        println!();
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let context = VulkanContext::new()?;
    inspect_devices(context.instance())
}
