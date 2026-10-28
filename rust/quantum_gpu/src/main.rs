use ash::{Entry, vk};
use std::error::Error;
use std::ffi::{CStr, CString};

fn verify_compute_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
) -> Result<(), Box<dyn Error>> {
    let priorities = [1.0_f32];

    let queue_info = [vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priorities)];

    // Enable 64-bit floating-point arithmetic for future shaders.
    let features = vk::PhysicalDeviceFeatures::default().shader_float64(true);

    let device_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queue_info)
        .enabled_features(&features);

    let device = unsafe { instance.create_device(physical_device, &device_info, None)? };

    let queue = unsafe { device.get_device_queue(queue_family_index, 0) };

    println!("  Logical device created");
    println!("  Compute queue handle: {queue:?}");

    let command_pool_info = vk::CommandPoolCreateInfo::default()
        .queue_family_index(queue_family_index)
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    let command_pool = unsafe { device.create_command_pool(&command_pool_info, None)? };

    let allocate_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);

    let command_buffers = unsafe { device.allocate_command_buffers(&allocate_info)? };

    let command_buffer = command_buffers[0];

    let begin_info =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    unsafe {
        device.begin_command_buffer(command_buffer, &begin_info)?;
        device.end_command_buffer(command_buffer)?;
    }

    let submit_buffers = [command_buffer];
    let submit_info = [vk::SubmitInfo::default().command_buffers(&submit_buffers)];

    unsafe {
        device.queue_submit(queue, &submit_info, vk::Fence::null())?;
        device.queue_wait_idle(queue)?;
    }

    println!("  Empty compute command buffer submitted successfully");

    unsafe {
        device.destroy_command_pool(command_pool, None);
        device.destroy_device(None);
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // Load the Vulkan loader dynamically.
    let entry = unsafe { Entry::load()? };

    let app_name = CString::new("quantum_gpu")?;
    let engine_name = CString::new("quantum_compute_lab")?;

    let app_info = vk::ApplicationInfo::default()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::API_VERSION_1_2);

    let create_info = vk::InstanceCreateInfo::default().application_info(&app_info);

    // Vulkan objects must be explicitly destroyed.
    let instance = unsafe { entry.create_instance(&create_info, None)? };

    let result = inspect_devices(&instance);

    unsafe {
        instance.destroy_instance(None);
    }

    result
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
