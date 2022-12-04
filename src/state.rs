use log::warn;
use wgpu::util::DeviceExt;
use winit::{
    window::Window,
    event::{KeyboardInput, WindowEvent, MouseButton},
};

use crate::vertex::{PureVertex, Vertex};
use crate::shader::create_spv_shader;
use crate::camera::{Camera, CameraUniform, CameraController, Projection};
use crate::texture::Texture;
use crate::ecs::{
    scene::Scene,
    component::mesh::MeshComponent,
    component::instance::{InstanceRaw, InstanceComponent},
};

#[cfg(target_os = "macos")]
pub const GRAPHICS_BACKEND: wgpu::Backends = wgpu::Backends::METAL;
#[cfg(not(target_os = "macos"))]
pub const GRAPHICS_BACKEND: wgpu::Backends = wgpu::Backends::VULKAN;
pub const DEVICE_FEATURES: wgpu::Features = wgpu::Features::POLYGON_MODE_LINE;
pub const DRAW_POLYGON_MODE: wgpu::PolygonMode = wgpu::PolygonMode::Fill;

pub struct State {
    // Rendering
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    pub camera_uniform: CameraUniform,
    pub camera_buffer: wgpu::Buffer,
    pub camera_bind_group: wgpu::BindGroup,
    pub camera: Camera,
    pub noise_storage_buffer: wgpu::Buffer,
    pub projection: Projection,
    pub camera_controller: CameraController,
    pub depth_texture: Texture,
    pub render_pipeline: wgpu::RenderPipeline,

    // Scenes
    pub scenes: Vec<Scene>,
    pub active_scene_index: usize,
}

impl State {
    pub async fn new(window: &Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        // Creating an instance is the prelude to creating Adapters
        // and Surfaces.
        let instance = wgpu::Instance::new(GRAPHICS_BACKEND);

        // The surface unto we draw.
        let surface = unsafe { instance.create_surface(window) };

        // Create handle to the graphics card. We use this to create
        // our Device and Queue later on.
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();

        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: DEVICE_FEATURES,
                limits: wgpu::Limits::default(),
                label: None,
            },
            None,
        ).await.unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let vertex_shader = create_spv_shader!(device, "../target/vertex.spv", "vertex");
        let fragment_shader = create_spv_shader!(device, "../target/fragment.spv", "fragment");

        let camera = Camera::new((0.0, 3.0, 6.0), cgmath::Deg(-90.0), cgmath::Deg(-20.0));
        let projection = Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 5000.0);
        let camera_controller = CameraController::new(32.0, 0.4);

        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });

        let noise_storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("noise_storage_buffer"),
            size: 256^3,
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });

        let depth_texture = Texture::create_depth_texture(&device, &config, "depth_texture");

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vertex_shader,
                entry_point: "main",
                buffers: &[
                    PureVertex::desc(), InstanceRaw::desc(),
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fragment_shader,
                entry_point: "main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: DRAW_POLYGON_MODE,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let scene = Scene::new();
        let scenes = vec![scene];
        let active_scene_index = 0;

        Self {
            surface,
            device,
            queue,
            config,
            size,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            camera,
            noise_storage_buffer,
            projection,
            camera_controller,
            depth_texture,
            render_pipeline,
            active_scene_index,
            scenes,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.depth_texture = Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
            self.projection.resize(new_size.width, new_size.height);
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            } => self.camera_controller.process_keyboard(*key, *state),

            WindowEvent::MouseWheel { delta, .. } => {
                self.camera_controller.process_scroll(delta);
                true
            },

            WindowEvent::MouseInput {
                button: MouseButton::Left,
                ..
            } => {
                true
            },

            _ => false,
        }
    }

    pub fn update(&mut self, dt: std::time::Duration) {
        self.camera_controller.update_camera(&mut self.camera, dt);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        // `begin_render_pass()' borrows encoder mutably (aka `&mut self'). We can't call
        // `encoder.finish()' until we release that mutable borrow, hence the block here.
        {
           let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    // This is what @location(0) in the fragment shader targets
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(
                                wgpu::Color {
                                    r: 0.0,
                                    g: 0.0,
                                    b: 0.0,
                                    a: 1.0,
                                }
                            ),
                            store: true,
                        }
                    })
                ],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for component in &self.get_active_scene().components {
                let mesh_component = match component.as_any().downcast_ref::<MeshComponent>() {
                    Some(c) => c,
                    None => continue,
                };

                render_pass.set_vertex_buffer(0, mesh_component.vertex_buffer.slice(..));

                let parent_object_components = &self.get_active_scene().objects.get(mesh_component.parent_index)
                    .expect("Invalid component parent index!").components;

                /* for sub_component_index in 0..parent_object_components.len() {
                        let sub_component = self.get_active_scene().components.get(sub_component_index)
                            .expect("Invalid sub component index!");
                        match sub_component.as_any().downcast_ref::<InstanceComponent>() {
                            Some(instance_component) => {
                                ;
                            },
                            None => {
                                error!("A mesh component also requires an instance component. Not rendering!");
                                continue;
                            },
                        }
                } */

                match &self.get_active_scene().components.get(mesh_component.instance_component_index)
                    .expect("Invalid instance component index (pointed to by mesh)!").as_any().downcast_ref::<InstanceComponent>() {
                        Some(instance_component) => render_pass.set_vertex_buffer(1, instance_component.instance_buffer.slice(..)),
                        None => {
                            warn!("Instance component pointed to by mesh component (via index {}) isn't actually an instance. Not rendering!",
                                mesh_component.instance_component_index);

                            continue;
                        }
                    }

                render_pass.set_index_buffer(mesh_component.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

                for sub_component_index in 0..parent_object_components.len() {
                        let sub_component = self.get_active_scene().components.get(sub_component_index)
                            .expect("Invalid sub component index!");
                        match sub_component.as_any().downcast_ref::<InstanceComponent>() {
                            Some(instance_component) => {
                                render_pass.draw_indexed(0..mesh_component.num_indices, 0, 0..instance_component.instances.len() as _);
                            },
                            None => {
                                // Regular drawing
                                render_pass.draw_indexed(0..mesh_component.num_indices, 0, 0..1);
                            },
                        }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    fn get_active_scene(&self) -> &Scene {
        self.scenes.get(self.active_scene_index)
            .expect(format!("Invalid active scene index ({})!", self.active_scene_index).as_str())
    }
}

