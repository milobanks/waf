use std::any::Any;
use wgpu::util::DeviceExt;

use crate::vertex::PureVertex;
use crate::ecs::component::Component;

pub struct MeshComponent {
    pub desc: String,
    pub vertices: Vec<PureVertex>,
    pub indices: Vec<u32>,
    pub vertex_buffer: wgpu::Buffer,
    pub num_vertices: u32,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub parent_index: usize,
    pub instance_component_index: usize,
}

impl Component for MeshComponent {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl MeshComponent {
    pub fn default(device: &wgpu::Device, parent_index: usize, instance_component_index: usize) -> Self {
        let vertices = vec![
            /* PureVertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.5, 1.0, 1.0] },
            PureVertex { position: [-0.49513406, 0.06958647, 0.0], color: [1.0, 0.0, 0.5] },
            PureVertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.25, 1.0, 0.5] },
            PureVertex { position: [0.35966998, -0.3473291, 0.0], color: [0.5, 0.0, 0.5] },
            PureVertex { position: [0.44147372, 0.2347359, 0.0], color: [1.0, 0.0, 1.0] }, */
            PureVertex { position: [0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [1.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },
            PureVertex { position: [0.0, 1.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [0.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [1.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [1.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },

            PureVertex { position: [1.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [2.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },
            PureVertex { position: [1.0, 1.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [1.0, 0.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [2.0, 0.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [2.0, 1.0, 0.0], color: [0.0, 1.0, 0.0] },

            PureVertex { position: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [1.0, 2.0, 0.0], color: [0.0, 1.0, 0.0] },
            PureVertex { position: [0.0, 2.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [0.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [1.0, 1.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [1.0, 2.0, 0.0], color: [0.0, 1.0, 0.0] },

            PureVertex { position: [1.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [2.0, 2.0, 0.0], color: [0.0, 1.0, 0.0] },
            PureVertex { position: [1.0, 2.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [1.0, 1.0, 0.0], color: [1.0, 0.0, 0.0] },
            PureVertex { position: [2.0, 1.0, 0.0], color: [0.0, 0.0, 1.0] },
            PureVertex { position: [2.0, 2.0, 0.0], color: [0.0, 1.0, 0.0] },
        ];

        let indices = vec![
            /* 0, 1, 4,
            1, 2, 4,
            2, 3, 4, */
            0, 1, 2,
            3, 4, 5,

            6, 7, 8,
            9, 10, 11,

            12, 13, 14,
            15, 16, 17,

            18, 19, 20,
            21, 22, 23,
        ];

        Self::new("DEFAULT".to_owned(), device, vertices, indices, parent_index, instance_component_index)
    }

    pub fn empty(device: &wgpu::Device) -> Self {
        Self::new("EMPTY".to_owned(), device, vec![], vec![], 0, 0)
    }

    pub fn new(desc: String, device: &wgpu::Device, vertices: Vec<PureVertex>, indices: Vec<u32>, parent_index: usize, instance_component_index: usize) -> Self {
        let buffers = Self::generate_buffers("UNINIT".to_owned(), &vertices, &indices, device);
        let vertex_buffer = buffers.0;
        let index_buffer = buffers.1;

        let num_vertices = vertices.len() as u32;
        let num_indices = indices.len() as u32;

        Self {
            desc,
            vertices,
            indices,
            vertex_buffer,
            num_vertices,
            index_buffer,
            num_indices,
            parent_index,
            instance_component_index,
        }
    }

    pub fn update_buffers(&mut self, device: &wgpu::Device) {
        let buffers = Self::generate_buffers(self.desc.clone(), &self.vertices, &self.indices, device);

        self.vertex_buffer = buffers.0;
        self.index_buffer = buffers.1;
    }

    fn generate_buffers(desc: String, vertices: &Vec<PureVertex>, indices: &Vec<u32>, device: &wgpu::Device) -> (wgpu::Buffer, wgpu::Buffer) {
        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(format!("Vertex Buffer ({})", desc).as_str()),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some(format!("Index Buffer ({})", desc).as_str()),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            }
        );

        return (vertex_buffer, index_buffer);
    }
}

