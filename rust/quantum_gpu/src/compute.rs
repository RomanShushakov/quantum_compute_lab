use ash::vk;
use std::error::Error;
use std::io::Cursor;
use std::path::PathBuf;

use crate::buffer::StorageBuffer;
use crate::context::ComputeDevice;

// Own the descriptor/pipeline objects so partial initialization is safe.
struct ComputePipeline<'a> {
    device: &'a ash::Device,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    shader_module: vk::ShaderModule,
}

impl<'a> ComputePipeline<'a> {
    fn new(
        device: &'a ash::Device,
        buffer: vk::Buffer,
        size: vk::DeviceSize,
        shader_name: &str,
        uniform: Option<&StorageBuffer<'_>>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut owned = Self {
            device,
            descriptor_set_layout: vk::DescriptorSetLayout::null(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_set: vk::DescriptorSet::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),
            shader_module: vk::ShaderModule::null(),
        };

        let mut bindings = vec![
            vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::COMPUTE),
        ];
        if uniform.is_some() {
            bindings.push(
                vk::DescriptorSetLayoutBinding::default()
                    .binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1)
                    .stage_flags(vk::ShaderStageFlags::COMPUTE),
            );
        }
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);
        owned.descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let mut pool_sizes = vec![
            vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::STORAGE_BUFFER)
                .descriptor_count(1),
        ];
        if uniform.is_some() {
            pool_sizes.push(
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(1),
            );
        }
        let pool_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(1);
        owned.descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None)? };

        let set_layouts = [owned.descriptor_set_layout];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(owned.descriptor_pool)
            .set_layouts(&set_layouts);
        owned.descriptor_set = unsafe { device.allocate_descriptor_sets(&allocate_info)? }[0];
        let buffer_info = [vk::DescriptorBufferInfo::default()
            .buffer(buffer)
            .offset(0)
            .range(size)];
        let uniform_info = uniform.map(|storage| {
            [vk::DescriptorBufferInfo::default()
                .buffer(storage.buffer())
                .offset(0)
                .range(storage.size())]
        });
        let mut writes = vec![
            vk::WriteDescriptorSet::default()
                .dst_set(owned.descriptor_set)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
                .buffer_info(&buffer_info),
        ];
        if let Some(ref uniform_info) = uniform_info {
            writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(owned.descriptor_set)
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(uniform_info),
            );
        }
        unsafe { device.update_descriptor_sets(&writes, &[]) };

        let pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        owned.pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        let shader_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("shaders")
            .join(shader_name);
        let shader_bytes = std::fs::read(&shader_path).map_err(|error| {
            format!(
                "cannot read {} ({error}); compile the WGSL shader with naga first",
                shader_path.display()
            )
        })?;
        let words = ash::util::read_spv(&mut Cursor::new(shader_bytes))?;
        let shader_info = vk::ShaderModuleCreateInfo::default().code(&words);
        owned.shader_module = unsafe { device.create_shader_module(&shader_info, None)? };

        let entry_name = c"main";
        let stage = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(owned.shader_module)
            .name(entry_name);
        let pipeline_info = [vk::ComputePipelineCreateInfo::default()
            .stage(stage)
            .layout(owned.pipeline_layout)];
        owned.pipeline = unsafe {
            device.create_compute_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
        }
        .map_err(|(_, error)| error)?[0];

        // The descriptor set lives as long as the pool; no separate free is required.
        Ok(owned)
    }
}

impl Drop for ComputePipeline<'_> {
    fn drop(&mut self) {
        unsafe {
            if self.pipeline != vk::Pipeline::null() {
                self.device.destroy_pipeline(self.pipeline, None);
            }
            if self.shader_module != vk::ShaderModule::null() {
                self.device.destroy_shader_module(self.shader_module, None);
            }
            if self.pipeline_layout != vk::PipelineLayout::null() {
                self.device
                    .destroy_pipeline_layout(self.pipeline_layout, None);
            }
            if self.descriptor_pool != vk::DescriptorPool::null() {
                self.device
                    .destroy_descriptor_pool(self.descriptor_pool, None);
            }
            if self.descriptor_set_layout != vk::DescriptorSetLayout::null() {
                self.device
                    .destroy_descriptor_set_layout(self.descriptor_set_layout, None);
            }
        }
    }
}

pub fn run_transform(
    compute: &ComputeDevice<'_>,
    storage: &StorageBuffer<'_>,
    count: usize,
) -> Result<(), Box<dyn Error>> {
    if count == 0 {
        return Err("cannot dispatch an empty buffer".into());
    }
    let byte_count = count
        .checked_mul(std::mem::size_of::<f32>())
        .ok_or("buffer size overflow")?;
    if byte_count as u64 > storage.size() {
        return Err("dispatch exceeds storage buffer size".into());
    }

    let device = compute.device();
    let pipeline = ComputePipeline::new(
        device,
        storage.buffer(),
        storage.size(),
        "transform.spv",
        None,
    )?;

    dispatch_compute(compute, &pipeline, count)
}

fn dispatch_compute(
    compute: &ComputeDevice<'_>,
    pipeline: &ComputePipeline<'_>,
    count: usize,
) -> Result<(), Box<dyn Error>> {
    let device = compute.device();
    let allocation = vk::CommandBufferAllocateInfo::default()
        .command_pool(compute.command_pool())
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let buffers = unsafe { device.allocate_command_buffers(&allocation)? };
    let command_buffer = buffers[0];

    let result = (|| -> Result<(), Box<dyn Error>> {
        let begin = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            device.begin_command_buffer(command_buffer, &begin)?;
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline,
            );
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::COMPUTE,
                pipeline.pipeline_layout,
                0,
                &[pipeline.descriptor_set],
                &[],
            );
            let groups = u32::try_from(count.div_ceil(64))?;
            device.cmd_dispatch(command_buffer, groups, 1, 1);

            // Make shader writes available to subsequent host reads.
            let barrier = [vk::MemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::HOST_READ)];
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::HOST,
                vk::DependencyFlags::empty(),
                &barrier,
                &[],
                &[],
            );
            device.end_command_buffer(command_buffer)?;
            let submitted_buffers = [command_buffer];
            let submits = [vk::SubmitInfo::default().command_buffers(&submitted_buffers)];
            device.queue_submit(compute.queue(), &submits, vk::Fence::null())?;
            device.queue_wait_idle(compute.queue())?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = unsafe { device.device_wait_idle() };
    }
    unsafe { device.free_command_buffers(compute.command_pool(), &buffers) };
    result
}

/// In-place Hadamard on interleaved [real, imaginary] f32 amplitudes.
/// The state vector must have a power-of-two number of complex amplitudes.
pub fn run_hadamard(
    compute: &ComputeDevice<'_>,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    storage: &StorageBuffer<'_>,
    amplitude_count: usize,
    target: u32,
) -> Result<(), Box<dyn Error>> {
    if amplitude_count < 2 || !amplitude_count.is_power_of_two() {
        return Err("amplitude count must be a power of two and at least 2".into());
    }
    if target >= amplitude_count.trailing_zeros() {
        return Err("Hadamard target qubit is out of range".into());
    }
    let required_bytes = amplitude_count
        .checked_mul(2)
        .and_then(|n| n.checked_mul(std::mem::size_of::<f32>()))
        .ok_or("state vector size overflow")?;
    if required_bytes as u64 > storage.size() {
        return Err("state vector exceeds storage buffer size".into());
    }
    let amplitude_count_u32 = u32::try_from(amplitude_count)?;
    let pairs = amplitude_count / 2;
    let params = StorageBuffer::new_with_usage(
        instance,
        physical_device,
        compute.device(),
        8,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
    )?;
    params.write_u32(&[target, amplitude_count_u32])?;
    let pipeline = ComputePipeline::new(
        compute.device(),
        storage.buffer(),
        required_bytes as vk::DeviceSize,
        "hadamard.spv",
        Some(&params),
    )?;
    dispatch_compute(compute, &pipeline, pairs)
}

/// Apply Ry(theta) to one qubit of an interleaved f32 complex state vector.
pub fn run_ry(
    compute: &ComputeDevice<'_>,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    storage: &StorageBuffer<'_>,
    amplitude_count: usize,
    target: u32,
    theta: f32,
) -> Result<(), Box<dyn Error>> {
    validate_gate_state(storage, amplitude_count, target)?;
    if !theta.is_finite() {
        return Err("Ry angle must be finite".into());
    }
    let count = u32::try_from(amplitude_count)?;
    let params = StorageBuffer::new_with_usage(
        instance,
        physical_device,
        compute.device(),
        16,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
    )?;
    let half = theta * 0.5;
    params.write_u32(&[target, count, half.cos().to_bits(), half.sin().to_bits()])?;
    let pipeline = ComputePipeline::new(
        compute.device(),
        storage.buffer(),
        (amplitude_count * 8) as u64,
        "ry.spv",
        Some(&params),
    )?;
    dispatch_compute(compute, &pipeline, amplitude_count / 2)
}

/// Apply controlled NOT using distinct control and target qubits.
pub fn run_cnot(
    compute: &ComputeDevice<'_>,
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    storage: &StorageBuffer<'_>,
    amplitude_count: usize,
    control: u32,
    target: u32,
) -> Result<(), Box<dyn Error>> {
    validate_gate_state(storage, amplitude_count, target)?;
    if control >= amplitude_count.trailing_zeros() || control == target {
        return Err("CNOT control qubit is invalid or equals target".into());
    }
    let count = u32::try_from(amplitude_count)?;
    let params = StorageBuffer::new_with_usage(
        instance,
        physical_device,
        compute.device(),
        16,
        vk::BufferUsageFlags::UNIFORM_BUFFER,
    )?;
    params.write_u32(&[control, target, count, 0])?;
    let pipeline = ComputePipeline::new(
        compute.device(),
        storage.buffer(),
        (amplitude_count * 8) as u64,
        "cnot.spv",
        Some(&params),
    )?;
    dispatch_compute(compute, &pipeline, amplitude_count / 2)
}

fn validate_gate_state(
    storage: &StorageBuffer<'_>,
    amplitude_count: usize,
    target: u32,
) -> Result<(), Box<dyn Error>> {
    if amplitude_count < 2 || !amplitude_count.is_power_of_two() {
        return Err("amplitude count must be a power of two and at least 2".into());
    }
    if target >= amplitude_count.trailing_zeros() {
        return Err("target qubit is out of range".into());
    }
    let bytes = amplitude_count
        .checked_mul(8)
        .ok_or("state vector size overflow")?;
    if bytes as u64 > storage.size() {
        return Err("state vector exceeds storage buffer size".into());
    }
    Ok(())
}
