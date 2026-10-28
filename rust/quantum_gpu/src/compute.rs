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

/// A gate in a circuit prepared once and submitted repeatedly.
/// Parameters and pipelines are immutable for the lifetime of the circuit.
#[derive(Clone, Copy, Debug)]
pub enum CircuitGate {
    Hadamard { target: u32 },
    Ry { target: u32, theta: f32 },
    Cnot { control: u32, target: u32 },
}

/// Prepared GPU circuit. The state buffer, uniforms, descriptors, pipelines,
/// and recorded command buffer are reused across submissions. All gates act
/// on the same GPU-resident state, without intermediate host readback.
///
/// This uses queue_wait_idle per execution; timing is submission-to-completion,
/// not GPU timestamp/kernel-only time.
pub struct PreparedCircuit<'a> {
    compute: &'a ComputeDevice<'a>,
    command_buffer: vk::CommandBuffer,
    // Vulkan objects must remain alive until after the command buffer finishes.
    _uniforms: Vec<StorageBuffer<'a>>,
    _pipelines: Vec<ComputePipeline<'a>>,
    timestamp_pool: vk::QueryPool,
    timestamp_period_ns: f64,
    timestamp_valid_bits: u32,
}

impl<'a> PreparedCircuit<'a> {
    pub fn new(
        compute: &'a ComputeDevice<'a>,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        storage: &StorageBuffer<'a>,
        amplitude_count: usize,
        gates: &[CircuitGate],
    ) -> Result<Self, Box<dyn Error>> {
        Self::new_internal(
            compute,
            instance,
            physical_device,
            storage,
            amplitude_count,
            gates,
            false,
        )
    }

    /// Request GPU timestamps when supported by the selected compute queue.
    /// Falls back to an untimed circuit if the queue exposes no timestamp bits.
    pub fn new_timed(
        compute: &'a ComputeDevice<'a>,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        storage: &StorageBuffer<'a>,
        amplitude_count: usize,
        gates: &[CircuitGate],
        queue_family_index: u32,
    ) -> Result<Self, Box<dyn Error>> {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
        let family = queue_families
            .get(queue_family_index as usize)
            .ok_or("invalid queue family index")?;
        let timestamp_bits = family.timestamp_valid_bits;
        let mut circuit = Self::new_internal(
            compute,
            instance,
            physical_device,
            storage,
            amplitude_count,
            gates,
            timestamp_bits > 0,
        )?;
        circuit.timestamp_valid_bits = timestamp_bits;
        circuit.timestamp_period_ns =
            unsafe { instance.get_physical_device_properties(physical_device) }
                .limits
                .timestamp_period as f64;
        Ok(circuit)
    }

    fn new_internal(
        compute: &'a ComputeDevice<'a>,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        storage: &StorageBuffer<'a>,
        amplitude_count: usize,
        gates: &[CircuitGate],
        timestamps: bool,
    ) -> Result<Self, Box<dyn Error>> {
        if gates.is_empty() {
            return Err("prepared circuit requires at least one gate".into());
        }
        validate_gate_state(storage, amplitude_count, 0)?;
        let count = u32::try_from(amplitude_count)?;
        let bytes = amplitude_count
            .checked_mul(8)
            .ok_or("state vector size overflow")? as u64;
        let mut uniforms = Vec::with_capacity(gates.len());
        let mut pipelines = Vec::with_capacity(gates.len());
        for &gate in gates {
            let (shader, params) = match gate {
                CircuitGate::Hadamard { target } => {
                    validate_gate_state(storage, amplitude_count, target)?;
                    ("hadamard.spv", vec![target, count])
                }
                CircuitGate::Ry { target, theta } => {
                    validate_gate_state(storage, amplitude_count, target)?;
                    if !theta.is_finite() {
                        return Err("Ry angle must be finite".into());
                    }
                    let half = theta * 0.5;
                    (
                        "ry.spv",
                        vec![target, count, half.cos().to_bits(), half.sin().to_bits()],
                    )
                }
                CircuitGate::Cnot { control, target } => {
                    validate_gate_state(storage, amplitude_count, target)?;
                    if control >= amplitude_count.trailing_zeros() || control == target {
                        return Err("invalid CNOT control/target".into());
                    }
                    ("cnot.spv", vec![control, target, count, 0])
                }
            };
            let uniform = StorageBuffer::new_with_usage(
                instance,
                physical_device,
                compute.device(),
                (params.len() * 4) as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
            )?;
            uniform.write_u32(&params)?;
            let pipeline = ComputePipeline::new(
                compute.device(),
                storage.buffer(),
                bytes,
                shader,
                Some(&uniform),
            )?;
            uniforms.push(uniform);
            pipelines.push(pipeline);
        }
        let device = compute.device();
        let timestamp_pool = if timestamps {
            let info = vk::QueryPoolCreateInfo::default()
                .query_type(vk::QueryType::TIMESTAMP)
                .query_count(2);
            unsafe { device.create_query_pool(&info, None)? }
        } else {
            vk::QueryPool::null()
        };
        let allocation = vk::CommandBufferAllocateInfo::default()
            .command_pool(compute.command_pool())
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let command_buffer = unsafe { device.allocate_command_buffers(&allocation)? }[0];
        let record_result = (|| -> Result<(), Box<dyn Error>> {
            unsafe {
                device
                    .begin_command_buffer(command_buffer, &vk::CommandBufferBeginInfo::default())?;
                if timestamp_pool != vk::QueryPool::null() {
                    device.cmd_reset_query_pool(command_buffer, timestamp_pool, 0, 2);
                    device.cmd_write_timestamp(
                        command_buffer,
                        vk::PipelineStageFlags::COMPUTE_SHADER,
                        timestamp_pool,
                        0,
                    );
                }
                for (index, pipeline) in pipelines.iter().enumerate() {
                    if index != 0 {
                        // Ensure the previous shader writes are visible to the next gate.
                        let barrier = [vk::MemoryBarrier::default()
                            .src_access_mask(vk::AccessFlags::SHADER_WRITE)
                            .dst_access_mask(
                                vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
                            )];
                        device.cmd_pipeline_barrier(
                            command_buffer,
                            vk::PipelineStageFlags::COMPUTE_SHADER,
                            vk::PipelineStageFlags::COMPUTE_SHADER,
                            vk::DependencyFlags::empty(),
                            &barrier,
                            &[],
                            &[],
                        );
                    }
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
                    device.cmd_dispatch(
                        command_buffer,
                        u32::try_from((amplitude_count / 2).div_ceil(64))?,
                        1,
                        1,
                    );
                }
                if timestamp_pool != vk::QueryPool::null() {
                    device.cmd_write_timestamp(
                        command_buffer,
                        vk::PipelineStageFlags::COMPUTE_SHADER,
                        timestamp_pool,
                        1,
                    );
                }
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
            }
            Ok(())
        })();
        if let Err(error) = record_result {
            unsafe {
                device.free_command_buffers(compute.command_pool(), &[command_buffer]);
                if timestamp_pool != vk::QueryPool::null() {
                    device.destroy_query_pool(timestamp_pool, None);
                }
            }
            return Err(error);
        }
        Ok(Self {
            compute,
            command_buffer,
            _uniforms: uniforms,
            _pipelines: pipelines,
            timestamp_pool,
            timestamp_period_ns: 0.0,
            timestamp_valid_bits: 0,
        })
    }

    /// Returns None when timestamps are unsupported; otherwise elapsed GPU time
    /// in microseconds. The query brackets dispatches and shader barriers only.
    pub fn gpu_elapsed_us(&self) -> Result<Option<f64>, Box<dyn Error>> {
        if self.timestamp_pool == vk::QueryPool::null() {
            return Ok(None);
        }
        let mut ticks = [0_u64; 2];
        unsafe {
            self.compute.device().get_query_pool_results(
                self.timestamp_pool,
                0,
                &mut ticks,
                vk::QueryResultFlags::TYPE_64 | vk::QueryResultFlags::WAIT,
            )?;
        }
        let mask = if self.timestamp_valid_bits == 64 {
            u64::MAX
        } else {
            (1_u64 << self.timestamp_valid_bits) - 1
        };
        let elapsed_ticks = ticks[1].wrapping_sub(ticks[0]) & mask;
        Ok(Some(
            elapsed_ticks as f64 * self.timestamp_period_ns / 1_000.0,
        ))
    }

    pub fn execute(&self) -> Result<(), Box<dyn Error>> {
        let device = self.compute.device();
        let buffers = [self.command_buffer];
        let submits = [vk::SubmitInfo::default().command_buffers(&buffers)];
        unsafe {
            device.queue_submit(self.compute.queue(), &submits, vk::Fence::null())?;
            device.queue_wait_idle(self.compute.queue())?;
        }
        Ok(())
    }
}

impl Drop for PreparedCircuit<'_> {
    fn drop(&mut self) {
        let device = self.compute.device();
        let _ = unsafe { device.device_wait_idle() };
        unsafe {
            device.free_command_buffers(self.compute.command_pool(), &[self.command_buffer]);
            if self.timestamp_pool != vk::QueryPool::null() {
                device.destroy_query_pool(self.timestamp_pool, None);
            }
        }
    }
}
