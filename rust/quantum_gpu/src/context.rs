use ash::{Entry, vk};
use std::error::Error;
use std::ffi::CString;

pub struct VulkanContext {
    // The entry must remain alive while its instance and device are in use.
    _entry: Entry,
    instance: ash::Instance,
}

impl VulkanContext {
    pub fn new() -> Result<Self, Box<dyn Error>> {
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
        let instance = unsafe { entry.create_instance(&create_info, None)? };
        Ok(Self {
            _entry: entry,
            instance,
        })
    }

    pub fn instance(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) };
    }
}

pub struct ComputeDevice<'a> {
    device: ash::Device,
    queue: vk::Queue,
    command_pool: vk::CommandPool,
    // The borrow ensures that the Vulkan instance outlives its logical device.
    _instance: &'a ash::Instance,
}

impl<'a> ComputeDevice<'a> {
    pub fn new(
        instance: &'a ash::Instance,
        physical_device: vk::PhysicalDevice,
        queue_family_index: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let priorities = [1.0_f32];
        let queue_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&priorities)];
        let features = vk::PhysicalDeviceFeatures::default().shader_float64(true);
        let device_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_info)
            .enabled_features(&features);
        let device = unsafe { instance.create_device(physical_device, &device_info, None)? };
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        let command_pool = match unsafe { device.create_command_pool(&command_pool_info, None) } {
            Ok(pool) => pool,
            Err(error) => {
                unsafe { device.destroy_device(None) };
                return Err(error.into());
            }
        };
        Ok(Self {
            device,
            queue,
            command_pool,
            _instance: instance,
        })
    }

    pub fn device(&self) -> &ash::Device {
        &self.device
    }

    pub fn queue(&self) -> vk::Queue {
        self.queue
    }

    pub fn command_pool(&self) -> vk::CommandPool {
        self.command_pool
    }

    pub fn submit_empty(&self) -> Result<(), Box<dyn Error>> {
        let allocate_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(self.command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let buffers = unsafe { self.device.allocate_command_buffers(&allocate_info)? };
        let command_buffer = buffers[0];
        let result = (|| -> Result<(), Box<dyn Error>> {
            let begin_info = vk::CommandBufferBeginInfo::default()
                .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            unsafe {
                self.device
                    .begin_command_buffer(command_buffer, &begin_info)?;
                self.device.end_command_buffer(command_buffer)?;
                let submit_buffers = [command_buffer];
                let submit_info = [vk::SubmitInfo::default().command_buffers(&submit_buffers)];
                self.device
                    .queue_submit(self.queue, &submit_info, vk::Fence::null())?;
                self.device.queue_wait_idle(self.queue)?;
            }
            Ok(())
        })();
        // Ensure submission is finished even if queue_wait_idle returned an error.
        if result.is_err() {
            let _ = unsafe { self.device.device_wait_idle() };
        }
        unsafe {
            self.device
                .free_command_buffers(self.command_pool, &buffers)
        };
        result
    }
}

impl Drop for ComputeDevice<'_> {
    fn drop(&mut self) {
        // No resources may be destroyed while GPU work is still using them.
        let _ = unsafe { self.device.device_wait_idle() };
        unsafe {
            self.device.destroy_command_pool(self.command_pool, None);
            self.device.destroy_device(None);
        }
    }
}
