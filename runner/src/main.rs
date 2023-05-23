use pollster::FutureExt;
use wgpu::util::DeviceExt;
use winit::{
    event::{Event, WindowEvent, ElementState, KeyboardInput, VirtualKeyCode},
    event_loop::{EventLoop},
    window::WindowBuilder, dpi::PhysicalSize,
};

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("WGPU Test")
        .with_inner_size(PhysicalSize { width: 1024, height: 768 })
        .build(&event_loop)
        .unwrap();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::PRIMARY),
        dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default()
    });
    let surface = unsafe { instance.create_surface(&window).unwrap() };
    let adapter = instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }
    ).block_on().unwrap();

    let (device, queue) = adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: None,
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
        },
        None,
    ).block_on().unwrap();

    let size = window.inner_size();
    let surface_config = wgpu::SurfaceConfiguration {
        usage: /*wgpu::TextureUsages::RENDER_ATTACHMENT |*/ wgpu::TextureUsages::COPY_DST,
        present_mode: wgpu::PresentMode::Fifo,
        ..surface.get_default_config(&adapter, size.width, size.height).unwrap()
    };
    surface.configure(&device, &surface_config);

    let shader2 = device.create_shader_module(wgpu::include_wgsl!("../../shader.wgsl"));
    let shader = device.create_shader_module(wgpu::include_spirv!(env!("shader_rust.spv")));

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                count: None,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                },
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                count: None,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    ty: wgpu::BufferBindingType::Uniform,
                },
            }
        ],
    });

    let compute_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&compute_pipeline_layout),
        module: &shader,
        entry_point: "cs_main",
    });

    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: size.width as u64 * size.height as u64 * 4,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_DST
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let mut camera: [f32; 3] = [1.0, 0.0, 0.0];
    let mut x_angle: f32 = 0.0;

    let camera_buffer = device.create_buffer_init(
        &wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&camera),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        }
    );

    let compute_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: camera_buffer.as_entire_binding(),
            }
        ],
    });

    event_loop.run(move |event, _, control_flow| {
        control_flow.set_wait();

        match event {
            Event::WindowEvent {
                window_id,
                event: WindowEvent::CloseRequested,
            } if window_id == window.id() => control_flow.set_exit(),

            Event::WindowEvent {
                window_id,
                event: WindowEvent::Resized(_new_size),
            } if window_id == window.id() => {
                //size = new_size;
                window.request_redraw()
            }

            Event::WindowEvent {
                window_id,
                event: WindowEvent::KeyboardInput { 
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::Q),
                        ..
                    },
                    ..
                },
            } if window_id == window.id() => {
                x_angle += 0.1;
                camera[0] = x_angle.cos();
                camera[1] = x_angle.sin();
                queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&camera));
                window.request_redraw()
            }

            Event::WindowEvent {
                window_id,
                event: WindowEvent::KeyboardInput { 
                    input: KeyboardInput {
                        state: ElementState::Pressed,
                        virtual_keycode: Some(VirtualKeyCode::E),
                        ..
                    },
                    ..
                },
            } if window_id == window.id() => {
                x_angle -= 0.1;
                camera[0] = x_angle.cos();
                camera[1] = x_angle.sin();
                queue.write_buffer(&camera_buffer, 0, bytemuck::cast_slice(&camera));
                window.request_redraw()
            }

            Event::RedrawRequested(_) => {
                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
                let output = surface.get_current_texture().unwrap();

                {
                    let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
                    cpass.set_bind_group(0, &compute_bind_group, &[]);
                    cpass.set_pipeline(&compute_pipeline);
                    cpass.dispatch_workgroups(size.width, size.height, 1);
                }

                encoder.copy_buffer_to_texture(
                    wgpu::ImageCopyBuffer {
                        buffer: &storage_buffer,
                        layout: wgpu::ImageDataLayout {
                            offset: 0,
                            bytes_per_row: Some(4 * size.width),
                            rows_per_image: None,
                        },
                    }, 
                    wgpu::ImageCopyTexture {
                        texture: &output.texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    output.texture.size()
                );
                
                queue.submit(Some(encoder.finish()));
                output.present();
            }
            _ => {}
        }
    })
}
