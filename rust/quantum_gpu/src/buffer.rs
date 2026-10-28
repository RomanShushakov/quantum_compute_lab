use ash::vk;
use std::error::Error;

pub struct StorageBuffer<'a> {
    device: &'a ash::Device,
    buffer: vk::Buffer,
    memory: vk::DeviceMemory,
    size: vk::DeviceSize,
    memory_type_index: u32,
}

impl<'a> StorageBuffer<'a> {
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &'a ash::Device,
        size: vk::DeviceSize,
    ) -> Result<Self, Box<dyn Error>> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(vk::BufferUsageFlags::STORAGE_BUFFER)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
        let buffer = unsafe { device.create_buffer(&buffer_info, None)? };
        let result = (|| -> Result<(vk::DeviceMemory, u32), Box<dyn Error>> {
            let requirements = unsafe { device.get_buffer_memory_requirements(buffer) };
            let properties =
                unsafe { instance.get_physical_device_memory_properties(physical_device) };
            let required =
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT;
            let index = (0..properties.memory_type_count)
                .filter(|&index| requirements.memory_type_bits & (1_u32 << index) != 0)
                .filter(|&index| {
                    properties.memory_types[index as usize]
                        .property_flags
                        .contains(required)
                })
                .max_by_key(|&index| {
                    let flags = properties.memory_types[index as usize].property_flags;
                    (
                        flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL),
                        flags.contains(vk::MemoryPropertyFlags::HOST_CACHED),
                    )
                })
                .ok_or("no compatible host-visible, host-coherent memory type")?;
            let allocation = vk::MemoryAllocateInfo::default()
                .allocation_size(requirements.size)
                .memory_type_index(index);
            let memory = unsafe { device.allocate_memory(&allocation, None)? };
            if let Err(error) = unsafe { device.bind_buffer_memory(buffer, memory, 0) } {
                unsafe { device.free_memory(memory, None) };
                return Err(error.into());
            }
            Ok((memory, index))
        })();
        match result {
            Ok((memory, memory_type_index)) => Ok(Self {
                device,
                buffer,
                memory,
                size,
                memory_type_index,
            }),
            Err(error) => {
                unsafe { device.destroy_buffer(buffer, None) };
                Err(error)
            }
        }
    }

    pub fn memory_type_index(&self) -> u32 {
        self.memory_type_index
    }

    pub fn buffer(&self) -> vk::Buffer {
        self.buffer
    }

    pub fn size(&self) -> vk::DeviceSize {
        self.size
    }

    pub fn write_f32(&self, values: &[f32]) -> Result<(), Box<dyn Error>> {
        self.check_size(values.len())?;
        let mapped = MappedMemory::new(self.device, self.memory, self.size)?;
        unsafe {
            std::ptr::copy_nonoverlapping(values.as_ptr(), mapped.ptr.cast::<f32>(), values.len());
        }
        Ok(())
    }

    pub fn read_f32(&self, count: usize) -> Result<Vec<f32>, Box<dyn Error>> {
        self.check_size(count)?;
        let mapped = MappedMemory::new(self.device, self.memory, self.size)?;
        let mut values = vec![0.0_f32; count];
        unsafe {
            std::ptr::copy_nonoverlapping(mapped.ptr.cast::<f32>(), values.as_mut_ptr(), count);
        }
        Ok(values)
    }

    fn check_size(&self, count: usize) -> Result<(), Box<dyn Error>> {
        let bytes = count
            .checked_mul(std::mem::size_of::<f32>())
            .ok_or("buffer length overflow")?;
        if bytes as u64 > self.size {
            return Err("values exceed storage buffer size".into());
        }
        Ok(())
    }
}

impl Drop for StorageBuffer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_buffer(self.buffer, None);
            self.device.free_memory(self.memory, None);
        }
    }
}

struct MappedMemory<'a> {
    device: &'a ash::Device,
    memory: vk::DeviceMemory,
    ptr: *mut std::ffi::c_void,
}

impl<'a> MappedMemory<'a> {
    fn new(
        device: &'a ash::Device,
        memory: vk::DeviceMemory,
        size: vk::DeviceSize,
    ) -> Result<Self, vk::Result> {
        let ptr = unsafe { device.map_memory(memory, 0, size, vk::MemoryMapFlags::empty())? };
        Ok(Self {
            device,
            memory,
            ptr,
        })
    }
}

impl Drop for MappedMemory<'_> {
    fn drop(&mut self) {
        unsafe { self.device.unmap_memory(self.memory) };
    }
}
