use vulkano::{
    VulkanLibrary,
    instance::{Instance, InstanceCreateInfo},
    device::{Device, DeviceCreateInfo, Features, QueueCreateInfo},
    buffer::{BufferUsage, CpuAccessibleBuffer},
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

    // Create a source buffer filled with CPU-provided data
    let source_data = Matrix4x4::identity();
    let source_buffer = CpuAccessibleBuffer::from_data(
        device.clone(), // Since device is an Arc<Device>, this is cheap
        BufferUsage { transfer_src: true, ..Default::default() },
        false,
        source_data,
    ).unwrap();

    // Create destination buffer
    let destination_data = Matrix4x4::default();
    let destination_buffer = CpuAccessibleBuffer::from_data(
        device.clone(),
        BufferUsage { transfer_dst: true, ..Default::default() },
        false,
        destination_data
    ).unwrap();

    // Now we need to create a command buffer to tell the GPU what to do
    // Create a command buffer builder
    let mut command_builder = AutoCommandBufferBuilder::primary(
        device.clone(),
        queue_family_index, // The queue to submit the command buffer to
        CommandBufferUsage::OneTimeSubmit
    ).unwrap();

    // Populate the command buffer with a command to copy the buffer data over
    command_builder.copy_buffer(CopyBufferInfo::buffers(
        source_buffer.clone(),
        destination_buffer.clone()
    )).unwrap();

    // Build the actual command buffer now
    let command_buffer = command_builder.build().unwrap();

    // Finally: submit the command buffer to the GPU for immediate execution
    let future = sync::now(device.clone()) // Make a "future" object to manage resources
        .then_execute(queue.clone(), command_buffer) // Queue up the commands
        .unwrap()
        .then_signal_fence_and_flush() // Set up fence, then flush
        .unwrap();

    future.wait(None).unwrap(); // Wait for fence so the next .read() won't fail

    println!("Source buffer: {:?}", source_buffer.read().unwrap().clone());
    println!("Destination buffer: {:?}", destination_buffer.read().unwrap().clone());
}
