use std::any::Any;
use cgmath::prelude::*;
use wgpu::util::DeviceExt;

use crate::ecs::component::Component;

const DEFAULT_INSTANCES_PER_ROW: u32 = 10;
pub const SINGLE_INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(0.0, 0.0, 0.0);
pub const FANCY_MULTI_INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(DEFAULT_INSTANCES_PER_ROW
                                                                        as f32 * 0.5, 0.0,DEFAULT_INSTANCES_PER_ROW as f32 * 0.5);

pub struct InstanceComponent {
    pub num_instances_per_row: u32,
    pub instance_displacement: cgmath::Vector3<f32>,
    pub instance_buffer: wgpu::Buffer,
    pub instances: Vec<Instance>,
}

pub struct Instance {
    pub position: cgmath::Vector3<f32>,
    pub rotation: cgmath::Quaternion<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
        }
    }
}

impl InstanceRaw {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    // While our vertex shader only uses locations 0, and 1 now, in later tutorials we'll
                    // be using 2, 3, and 4, for Vertex. We'll start at slot 5 not conflict with them later
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x4,
                },
                // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
                // for each vec4. We'll have to reassemble the mat4 in
                // the shader.
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }
}

impl Component for InstanceComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl InstanceComponent {
    pub fn default(device: &wgpu::Device) -> Self {
        Self::new(device, 0, SINGLE_INSTANCE_DISPLACEMENT)
    }

    pub fn new(device: &wgpu::Device, num_instances_per_row: u32, instance_displacement: cgmath::Vector3<f32>) -> Self {
        let instances = Self::create_instances(num_instances_per_row, instance_displacement);
        let instance_buffer = Self::create_instance_buffer(device, &instances);

        Self {
            num_instances_per_row,
            instance_displacement,
            instance_buffer,
            instances,
        }
    }

    fn create_instance_buffer(device: &wgpu::Device, instances: &Vec<Instance>) -> wgpu::Buffer {
        let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();

        device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&instance_data),
                usage: wgpu::BufferUsages::VERTEX,
            }
        )
    }

    fn create_instances(num_instances_per_row: u32, instance_displacement: cgmath::Vector3<f32>) -> Vec<Instance> {
        (0..num_instances_per_row).flat_map(|z| {
            (0..num_instances_per_row).map(move |x| {
                let position = cgmath::Vector3 { x: x as f32, y: 0.0, z: z as f32 } - instance_displacement;

                let rotation = if position.is_zero() {
                    // This is needed so an object at (0, 0, 0) won't get scaled to zero
                    // as Quaternions can effect scale if they're not created correctly
                    cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
                } else {
                    cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
                };

                Instance {
                    position, rotation,
                }
            })
        }).collect::<Vec<_>>()
    }
}

