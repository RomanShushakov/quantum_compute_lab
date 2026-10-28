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

        let binding = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::COMPUTE)];
        let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&binding);
        owned.descriptor_set_layout =
            unsafe { device.create_descriptor_set_layout(&layout_info, None)? };

        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::STORAGE_BUFFER)
            .descriptor_count(1)];
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
        let writes = [vk::WriteDescriptorSet::default()
            .dst_set(owned.descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::STORAGE_BUFFER)
            .buffer_info(&buffer_info)];
        unsafe { device.update_descriptor_sets(&writes, &[]) };

        let pipeline_layout_info =
            vk::PipelineLayoutCreateInfo::default().set_layouts(&set_layouts);
        owned.pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None)? };

        // Compile with: naga quantum_gpu/shaders/transform.wgsl quantum_gpu/shaders/transform.spv
        let shader_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("shaders/transform.spv");
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
    let pipeline = ComputePipeline::new(device, storage.buffer(), storage.size())?;

    dispatch_transform(compute, &pipeline, count)
}

fn dispatch_transform(
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
