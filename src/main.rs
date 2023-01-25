use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateInfo},
    device::{Device, DeviceCreateInfo, Features, QueueCreateInfo},
    buffer::{BufferUsage, CpuAccessibleBuffer, TypedBufferAccess},
    command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, CopyBufferInfo},
    sync::{self, GpuFuture}
};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Zeroable, Pod)]
struct Matrix4x4 {
    x: [f32; 4],
    y: [f32; 4],
    z: [f32; 4],
    w: [f32; 4]
}

impl Matrix4x4 {
    fn identity() -> Self {
        Self {
            x: [1.0, 0.0, 0.0, 0.0],
            y: [0.0, 1.0, 0.0, 0.0],
            z: [0.0, 0.0, 1.0, 0.0],
            w: [0.0, 0.0, 0.0, 1.0]
        }
    }
}

fn main() {
    // Get library and create instance
    let library = VulkanLibrary::new().unwrap();
    let instance = Instance::new(library, InstanceCreateInfo::default()).unwrap();

    // Grab first device we can find
    let physical = instance.enumerate_physical_devices()
                           .unwrap()
                           .next()
                           .unwrap();

    // Get the index of a viable queue family for graphics
    let queue_family_index = physical.queue_family_properties()
                                     .iter()
                                     .enumerate()
                                     .position(|(_, q)| q.queue_flags.graphics)
                                     .unwrap() as u32;

    // Create a virtual device and get an iterator of queues for it
    let (device, mut queues) = Device::new(
        physical,
        DeviceCreateInfo {
            queue_create_infos: vec![
                QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }
            ],
            ..Default::default()
        }
    ).unwrap();

    // Since we only actually requested/need one queue, isolate it
    let queue = queues.next().unwrap();

    let data_iter = 0..65536;
    let data_buffer = CpuAccessibleBuffer::from_iter(device.clone(), BufferUsage {
        storage_buffer: true,
        ..Default::default()
    }, false, data_iter).unwrap();
}

mod cs {
    vulkano_shaders::shader!{
        ty: "compute",
        src: "
#version 450

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Data {
    uint data[];
} buf;

void main() {
    uint idx = gl_GlobalInvocationID.x;
    buf.data[idx] *= 12;
}"
    }
}
