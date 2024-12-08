// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

// use cgmath::{perspective, Deg, Matrix4, Point3, Rad, SquareMatrix, Vector2, Vector3, Vector4, Zero};
use std::{borrow::Cow, rc::Rc};
use glam::{Mat4, Vec2, Vec3};
use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct AVertex {
    position: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
}

impl AVertex {
    fn new(position: Vec3, color: wgpu::Color, uv: Vec2) -> AVertex {
        AVertex {
            position: position.into(),
            color: [color.r as f32, color.g as f32, color.b as f32, color.a as f32],
            uv: [uv.x, uv.y],
        }
    }
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<AVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x4,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 7]>() as wgpu::BufferAddress,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MatrixUniform {
    matrix: [[f32; 4]; 4],
}

impl MatrixUniform {
    fn from(mtx: &Mat4) -> MatrixUniform {
        MatrixUniform {
            matrix: (*mtx).to_cols_array_2d(),
        }
    }
}

pub fn parallelogram(
    position: Vec3,
    edge1: Vec3,
    edge2: Vec3,
    uv_position: Vec2,
    uv_edge1: Vec2,
    uv_edge2: Vec2,
    color: wgpu::Color,
) -> ([AVertex; 4], [u16; 6]) {
    (
        [
            AVertex::new(position, color, uv_position),
            AVertex::new(position + edge1, color, uv_position + uv_edge1),
            AVertex::new(position + edge1 + edge2, color, uv_position + uv_edge1 + uv_edge2),
            AVertex::new(position + edge2, color, uv_position + uv_edge2),
        ],
        [0, 1, 2, 0, 2, 3]
    )
}

pub fn rectangle(
    position: Vec3,
    width: f32,
    height: f32,
    uv_position: Vec2,
    uv_width: f32,
    uv_height: f32,
    color: wgpu::Color,
) -> ([AVertex; 4], [u16; 6]) {
    parallelogram(
        position,
        width * Vec3::X,
        height * Vec3::Y,
        uv_position,
        uv_width * Vec2::X,
        uv_height * Vec2::Y,
        color,
    )
}

pub trait Camera {
    fn matrix(&self, screen: &wgpu::SurfaceConfiguration) -> Mat4;
    fn texture(&self) -> Option<Rc<wgpu::TextureView>>;
}

#[derive(Debug)]
pub struct Camera2D {
    pub rotation: f32,
    pub zoom: Vec2,
    pub target: Vec2,
    pub offset: Vec2,
    pub texture: Option<Rc<wgpu::TextureView>>,
}

#[derive(Debug)]
pub struct Camera3D {
    pub position: Vec3,
    pub target: Vec3,
    pub up: Vec3,
    pub fov_y: f32,
    pub texture: Option<Rc<wgpu::TextureView>>,
}

impl Camera2D {
    pub fn from_rect(position: Vec2, size: Vec2, texture: Option<Rc<wgpu::TextureView>>) -> Camera2D {
        let target = position + (size / 2.);

        Camera2D {
            target,
            zoom: Vec2::new(1. / size.x * 2., -1. / size.y * 2.),
            offset: Vec2::ZERO,
            rotation: 0.,
            texture,
        }
    }
}

impl Camera for Camera2D {
    fn matrix(&self, _screen: &wgpu::SurfaceConfiguration) -> Mat4 {
        let mat_origin = Mat4::from_translation(Vec3::new(-self.target.x, -self.target.y, 0.0));
        let mat_rotation = Mat4::from_axis_angle(Vec3::new(0.0, 0.0, 1.0), self.rotation);

        let y_invert = if self.texture.is_some() { -1.0 } else { 1.0 };

        let mat_scale = Mat4::from_scale(Vec3::new(self.zoom.x, self.zoom.y * y_invert, 1.0));
        let mat_translation = Mat4::from_translation(Vec3::new(self.offset.x, self.offset.y, 0.0));

        mat_translation * ((mat_scale * mat_rotation) * mat_origin)
    }
    fn texture(&self) -> Option<Rc<wgpu::TextureView>> {
        self.texture.clone()
    }
}

// #[rustfmt::skip]
// const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
//     1.0, 0.0, 0.0, 0.0,
//     0.0, 1.0, 0.0, 0.0,
//     0.0, 0.0, 0.5, 0.5,
//     0.0, 0.0, 0.0, 1.0,
// );

impl Camera for Camera3D {
    fn matrix(&self, screen: &wgpu::SurfaceConfiguration) -> Mat4 {
        let aspect = screen.width as f32 / screen.height as f32;

        let view = Mat4::look_at_rh(self.position, self.target, self.up);
        let proj = Mat4::perspective_rh_gl(self.fov_y, aspect, 0.01, 10000.0);

        return proj * view;
    }
    fn texture(&self) -> Option<Rc<wgpu::TextureView>> {
        self.texture.clone()
    }
}

impl Default for Camera3D {
    fn default() -> Camera3D {
        Camera3D {
            position: Vec3::new(0., 0., 35.),
            target: Vec3::new(0., 0., 0.),
            up: Vec3::Y,
            fov_y: 45.0_f32.to_radians(),
            texture: None,
        }
    }
}

pub struct State<'a> {
    frame_texture: Option<Rc<wgpu::TextureView>>,
    frame: Option<wgpu::SurfaceTexture>,

    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    render_pipeline: wgpu::RenderPipeline,
    texture_bind_group_layout: wgpu::BindGroupLayout,
    matrix_bind_group_layout: wgpu::BindGroupLayout,
    white_texture: Rc<wgpu::BindGroup>,

    active_render_pass: Option<(wgpu::CommandEncoder, wgpu::RenderPass<'static>)>,

    camera_matrix: Mat4,
    camera_texture: Option<Rc<wgpu::TextureView>>,
    active_bind_group: Rc<wgpu::BindGroup>,
    vertices: Vec<AVertex>,
    indices: Vec<u16>,
}

impl State<'_> {
    pub async fn new<'a>(window: &'a sdl2::video::Window) -> Result<State<'a>, String> {
        let (width, height) = window.size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::from_window(&window).unwrap())
                .map_err(|e| e.to_string())?
        };
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .ok_or("adapter not found")?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_limits: wgpu::Limits::default(),
                    label: Some("device"),
                    required_features: wgpu::Features::POLYGON_MODE_LINE | wgpu::Features::POLYGON_MODE_POINT,
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .map_err(|e| e.to_string())?;

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: Vec::default(),
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let matrix_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("matrix_bind_group_layout"),
            });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&texture_bind_group_layout, &matrix_bind_group_layout],
            push_constant_ranges: &[],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("ashader.wgsl"))),
        });

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[AVertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache: None,
        });

        let white_texture = Rc::new(State::white_texture(&device, &queue, &texture_bind_group_layout));
        let frame = surface.get_current_texture().map_err(|e| e.to_string())?;
        let output = Rc::new(frame.texture.create_view(&wgpu::TextureViewDescriptor::default()));

        Ok(State {
            surface,
            device,
            queue,
            config,
            render_pipeline,
            texture_bind_group_layout,
            matrix_bind_group_layout,
            white_texture: white_texture.clone(),

            frame: Some(frame),
            frame_texture: Some(output),

            active_render_pass: None,

            camera_matrix: Mat4::IDENTITY,
            camera_texture: None,
            active_bind_group: white_texture,
            vertices: vec![],
            indices: vec![],
        })
    }
    fn white_texture(device: &wgpu::Device, queue: &wgpu::Queue, texture_bind_group_layout: &wgpu::BindGroupLayout) -> wgpu::BindGroup {
        let size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };

       let texture = device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("blocks"),
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        texture_bind_group
    }
    pub fn create_texture(&self, width: u32, height: u32) -> (Rc<wgpu::BindGroup>, Rc<wgpu::TextureView>) {
        let size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

       let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("blocks"),
            view_formats: &[],
        });

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        (Rc::new(texture_bind_group), Rc::new(texture_view))
    }
    pub fn upload_texture(&self, surface: &sdl2::surface::Surface) -> Rc<wgpu::BindGroup> {
        let size = wgpu::Extent3d {
            width: surface.width(),
            height: surface.height(),
            depth_or_array_layers: 1,
        };

       let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("blocks"),
            view_formats: &[],
        });
        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            surface.without_lock().unwrap(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(surface.pitch()),
                rows_per_image: Some(surface.height()),
            },
            size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let texture_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("texture_bind_group"),
        });

        Rc::new(texture_bind_group)
    }
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.config.width = width as u32;
        self.config.height = height as u32;

        let frame = std::mem::replace(&mut self.frame, None).unwrap();
        let output = std::mem::replace(&mut self.frame_texture, None).unwrap();

        drop(frame);
        drop(output);

        self.surface.configure(&self.device, &self.config);

        let next_frame = self
            .surface
            .get_current_texture()
            .map_err(|e| e.to_string())?;

        let next_output = next_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.frame = Some(next_frame);
        self.frame_texture = Some(Rc::new(next_output));

        Ok(())
    }
    pub fn queue_draw<const V: usize, const I: usize>(&mut self, data: ([AVertex; V], [u16; I])) {
        let (v, i) = data;
        let count = self.vertices.len() as u16;
        self.indices.extend(i.iter().map(|x| *x + count));
        self.vertices.extend_from_slice(&v);
    }
    pub fn set_texture(&mut self, texture: Option<Rc<wgpu::BindGroup>>) {
        self.active_bind_group = texture.unwrap_or(self.white_texture.clone());
    }
    pub fn set_camera(&mut self, camera: &dyn Camera) {
        self.camera_matrix = camera.matrix(&self.config);
        self.camera_texture = camera.texture();
    }
    pub fn start_render_pass(&mut self, clear: Option<wgpu::Color>) {
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("command_encoder"),
            });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: match &self.camera_texture {
                    Some(texture) => &texture,
                    None => &self.frame_texture.as_ref().unwrap(),
                },
                resolve_target: None,
                ops: wgpu::Operations {
                    load: clear.map(wgpu::LoadOp::Clear).unwrap_or(wgpu::LoadOp::Load),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        }).forget_lifetime();
        pass.set_pipeline(&self.render_pipeline);

        self.active_render_pass = Some((encoder, pass));
    }
    pub fn complete_render_pass(&mut self) -> Result<(), String> {
        let (encoder, render_pass) = std::mem::replace(&mut self.active_render_pass, None).ok_or("tried to complete a render pass without one being active")?;

        drop(render_pass);
        self.queue.submit(std::iter::once(encoder.finish()));
        Ok(())
    }
    pub fn do_draw(&mut self) -> Result<(), String> {
        if self.vertices.is_empty() {
            return Ok(());
        }
        let (_, ref mut render_pass) = self.active_render_pass.as_mut().ok_or("tried to draw without a render pass being active")?;

        let matrix = MatrixUniform::from(&self.camera_matrix);

        let matrix_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Matrix Buffer"),
                contents: bytemuck::cast_slice(&[matrix]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let matrix_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.matrix_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: matrix_buffer.as_entire_binding(),
            }],
            label: Some("matrix_bind_group"),
        });

        let vertex_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Well Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Well Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let num_indices = self.indices.len() as u32;

        render_pass.set_bind_group(0, self.active_bind_group.as_ref(), &[]);
        render_pass.set_bind_group(1, &matrix_bind_group, &[]);
        render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        render_pass.draw_indexed(0..num_indices, 0, 0..1);

        self.vertices.clear();
        self.indices.clear();

        Ok(())
    }
    pub fn present(&mut self) -> Result<(), String> {
        let frame = std::mem::replace(&mut self.frame, None).unwrap();
        let _output = std::mem::replace(&mut self.frame_texture, None).unwrap();

        frame.present();

        let next_frame = self
            .surface
            .get_current_texture()
            .map_err(|e| e.to_string())?;

        let next_output = next_frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.frame = Some(next_frame);
        self.frame_texture = Some(Rc::new(next_output));

        Ok(())
    }
}
