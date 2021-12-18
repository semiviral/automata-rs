#![feature(
    raw_ref_op,
    const_fn_trait_bound,
    linked_list_cursors,
    once_cell,
    result_option_inspect
)]

use crate::opengl::{buffer::RingBuffer, OpenGLObject};
use specs::Builder;
use winit::{dpi::LogicalSize, event_loop::*, window::Window};

mod blocks;
mod chunks;
mod collections;
mod input;
mod logger;
mod memory;
mod opengl;
mod render;
mod ring;
mod time;

#[macro_use]
extern crate log;
extern crate gl;

const DEFAULT_VERTEX_SRC: &str = r#"
    #version 450 core

    layout (location = 0) in vec2 v_pos;

    out gl_PerVertex { vec4 gl_Position; };

    layout (location = 0) out vec3 a_color;

    void main() {
        a_color = vec3(0.3);
        gl_Position = vec4(v_pos, 0.0, 1.0);
    }
"#;

const DEFAULT_FRAGMENT_SRC: &str = r#"
    #version 450 core

    layout (location = 0) in vec3 a_color;

    out vec4 f_color;
    
    void main() {
       f_color = vec4(a_color, 1.0);
    }
"#;

const VERTICES: [f32; 18] = [
    0.5, -0.5, 0.0,
    0.5, 0.5, 0.0,
    -0.5, -0.5, 0.0,
    -0.5, 0.5, 0.0,
    -0.5, -0.5, 0.0,
    0.5, 0.5, 0.0,
];

const INDICES: [u32; 6] = [1, 2, 3, 2, 3, 1];

static mut FRAME_COUNTER: usize = 0;

pub fn get_frame_count() -> usize {
    unsafe { FRAME_COUNTER }
}

pub struct AutomataWindow {
    window: Window,
}

impl AutomataWindow {
    pub fn viewport(&self) -> glam::Vec4 {
        let inner_size = self.window.inner_size();
        glam::Vec4::new(0.0, 0.0, inner_size.width as f32, inner_size.height as f32)
    }

    pub fn aspect_ratio(&self) -> f32 {
        let inner_size = self.window.inner_size();
        (inner_size.width as f32) / (inner_size.height as f32)
    }

    pub fn set_title(&mut self, title: &str) {
        self.window.set_title(title);
    }
}

fn configure_environment() -> (
    EventLoop<()>,
    winit::window::Window,
    raw_gl_context::GlContext,
) {
    let event_loop = EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_title("Automata")
        .with_inner_size(LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)
        .unwrap();
    let gl_context = raw_gl_context::GlContext::create(
        &window,
        raw_gl_context::GlConfig {
            version: (4, 5),
            profile: raw_gl_context::Profile::Core,
            red_bits: 8,
            blue_bits: 8,
            green_bits: 8,
            alpha_bits: 0,
            depth_bits: 0,
            stencil_bits: 0,
            samples: None,
            srgb: true,
            double_buffer: true,
            vsync: true,
        },
    )
    .unwrap();
    gl_context.make_current();
    gl::load_with(|s| gl_context.get_proc_address(s) as *const _);

    // Initialize OpenGL.
    unsafe {
        let version = std::ffi::CStr::from_ptr(gl::GetString(gl::VERSION) as *const _);
        info!("OpenGL version string: {:?}", version);

        let mut flags = 0;
        gl::GetIntegerv(gl::CONTEXT_FLAGS, &raw mut flags);
        if ((flags as u32) & gl::CONTEXT_FLAG_DEBUG_BIT) == 0 {
            warn!(
                "OpenGL device does not support a debug context. Error reporting will be impacted."
            );
        } else {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
            todo!("Set up default debug output callback.");
        }

        gl::ClearColor(1.0, 0.5, 0.1, 1.0);
        gl::Enable(gl::DEPTH_TEST);
    }

    (event_loop, window, gl_context)
}

fn main() {
    log::set_max_level(log::LevelFilter::Debug);
    log::set_logger(&logger::LOGGER).unwrap();
    crate::blocks::REGISTRY.lazy_init();

    let (event_loop, window, gl_context) = configure_environment();

    use opengl::{
        buffer::{Buffer, BufferDraw},
        check_errors,
        shader::{Fragment, ProgramPipeline, ShaderProgram, Vertex},
        VertexArrayObject, VertexFormat,
    };

    let vertex_shader = ShaderProgram::<Vertex>::new(&[DEFAULT_VERTEX_SRC]);
    check_errors();
    let fragment_shader = ShaderProgram::<Fragment>::new(&[DEFAULT_FRAGMENT_SRC]);
    check_errors();
    let program_pipeline = ProgramPipeline::new(vertex_shader, fragment_shader);
    check_errors();
    let vertices_buffer = Buffer::<f32>::new_data(&VERTICES, BufferDraw::Static);
    check_errors();
    let indices_buffer = Buffer::<u32>::new_data(&INDICES, BufferDraw::Static);
    check_errors();
    let mut vao = VertexArrayObject::new();
    check_errors();
    vao.allocate_vertex_attribute(0, 3, 0, 0, VertexFormat::F32(false));
    check_errors();
    vao.allocate_vertex_buffer_binding(0, &vertices_buffer, 0, 0);
    check_errors();
    vao.commit(None);
    check_errors();

    use specs::{World, WorldExt};

    let mut world = specs::World::new();

    // Insert resources.
    world.insert(input::InputTracker::default());
    world.insert(input::InputEventQueue::default());
    world.insert(time::DeltaTime(std::time::Duration::ZERO));
    world.insert(AutomataWindow { window });

    let mut max_uniform_alignment = 0;
    unsafe {
        gl::GetIntegerv(
            gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT,
            &raw mut max_uniform_alignment,
        );
    }
    world.insert(RingBuffer::<crate::render::CameraUniforms>::new(
        3,
        max_uniform_alignment as usize,
    ));

    // Register components.
    world.register::<input::InputVector>();

    // Create entities.
    world
        .create_entity()
        .with(input::InputVector(glam::Vec2::ZERO))
        .build();

    // Register systems.
    let mut dispatcher = specs::DispatcherBuilder::new()
        .with(input::InputSystem, "input", &[])
        .with_barrier()
        .with_thread_local(render::MultiDrawIndirectRenderSystem::new())
        .build();

    let mut stopwatch = time::Stopwatch::new();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        use winit::event::{Event, WindowEvent};

        // Refresh the frame delta time.
        {
            let mut delta_time = world.write_resource::<time::DeltaTime>();
            delta_time.0 = stopwatch.elapsed();
            stopwatch.restart();
            world
                .write_resource::<AutomataWindow>()
                .set_title(format!("Automata FPS {}", 1.0 / delta_time.0.as_secs_f64()).as_str());
        }

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => unsafe {
                gl::Viewport(0, 0, size.width as i32, size.height as i32);
            },

            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        device_id,
                        input,
                        is_synthetic,
                    },
                ..
            } => {
                use winit::event::VirtualKeyCode;

                // Allow exiting the game with the ESC key.
                // TODO map this in an input config.

                if let Some(keycode) = input.virtual_keycode {
                    match keycode {
                        VirtualKeyCode::Escape => {
                            *control_flow = ControlFlow::Exit;
                        }

                        VirtualKeyCode::E => {
                            info!("GL ERROR CHECK: {}", unsafe { gl::GetError() });
                        }

                        _ => {}
                    }
                } else {
                    world
                        .write_resource::<input::InputEventQueue>()
                        .push_event(input);
                }
            }

            Event::MainEventsCleared => {
                unsafe {
                    check_errors();
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                    check_errors();
                    program_pipeline.bind();
                    check_errors();
                    vao.bind();
                    check_errors();
                    gl::DrawArrays(gl::TRIANGLES, 0, vertices_buffer.data_len() as i32);
                    check_errors();

                    gl_context.swap_buffers();
                }

                // dispatcher.dispatch(&mut world);
                // world.maintain();
            }

            _ => {}
        }
    })
}

// unsafe fn init() {
//     //log::set_max_level(log::LevelFilter::Debug);

//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//

//     let vertex_shader = gl::CreateShader
//         .create_shader(
//             ShaderStage::Vertex,
//             DEFAULT_VERTEX_SRC.as_bytes(),
//             ShaderFlags::VERBOSE,
//         )
//         .unwrap();
//     let fragment_shader = gl
//         .create_shader(
//             ShaderStage::Fragment,
//             DEFAULT_FRAGMENT_SRC.as_bytes(),
//             ShaderFlags::VERBOSE,
//         )
//         .unwrap();

//     let pipeline = gl
//         .create_graphics_pipeline(
//             grr::VertexPipelineDesc {
//                 vertex_shader,
//                 tessellation_control_shader: None,
//                 tessellation_evaluation_shader: None,
//                 geometry_shader: None,
//                 fragment_shader: Some(fragment_shader),
//             },
//             grr::PipelineFlags::VERBOSE,
//         )
//         .unwrap();

//     let vertex_array = gl
//         .create_vertex_array(&[
//             grr::VertexAttributeDesc {
//                 location: 0,
//                 binding: 0,
//                 format: grr::VertexFormat::Xyz32Float,
//                 offset: 0,
//             },
//             grr::VertexAttributeDesc {
//                 location: 1,
//                 binding: 0,
//                 format: grr::VertexFormat::Xyz32Float,
//                 offset: (2 * std::mem::size_of::<f32>()) as _,
//             },
//         ])
//         .unwrap();

//     let triangles = gl
//         .create_buffer_from_host(grr::as_u8_slice(&VERTICES), grr::MemoryFlags::empty())
//         .unwrap();

//     event_loop.run(update)
// }

// fn update(event: Event<()>, control_flow: &mut ControlFlow) {
//     *control_flow = ControlFlow::Poll;

//     use winit::event::Event;
//     match event {
//         // Capture keyboard input
//         Event::WindowEvent {
//             event:
//                 WindowEvent::KeyboardInput {
//                     device_id,
//                     input: keyboard_input,
//                     is_synthetic,
//                 },
//             ..
//         } => {
//             println!("{:?}", (device_id, keyboard_input, is_synthetic));
//             unsafe {
//                 KEY_EVENTS.push((device_id, keyboard_input, is_synthetic));
//             }
//         }

//         // Capture window closure
//         Event::WindowEvent {
//             event: WindowEvent::CloseRequested,
//             ..
//         } => {
//             println!("Force game exit requested.");
//             *control_flow = ControlFlow::Exit
//         }

//         Event::LoopDestroyed => unsafe {
//             let gl = gl();

//             gl.delete_shaders(&[vertex_shader, fragment_shader]);
//             gl.delete_pipeline(pipeline);
//             gl.delete_buffer(triangles);
//             gl.delete_vertex_array(vertex_array);
//         },

//         // Update()
//         Event::MainEventsCleared => unsafe {
//             KEY_EVENTS.clear();
//         },
//         _ => {}
//     }
// }

// unsafe fn render() {
//     let gl = gl();

//     let size = window.inner_size();

//     gl.bind_pipeline(pipeline);
//     gl.bind_vertex_array(vertex_array);
//     gl.bind_vertex_buffers(
//         vertex_array,
//         0,
//         &[grr::VertexBufferView {
//             buffer: triangles,
//             offset: 0,
//             stride: (5 * std::mem::size_of::<f32>()) as _,
//             input_rate: grr::InputRate::Vertex,
//         }],
//     );

//     gl.set_viewport(
//         0,
//         &[grr::Viewport {
//             x: 0.0,
//             y: 0.0,
//             w: size.width as _,
//             h: size.height as _,
//             n: 0.0,
//             f: 1.0,
//         }],
//     );

//     gl.set_scissor(
//         0,
//         &[grr::Region {
//             x: 0,
//             y: 0,
//             w: size.width as _,
//             h: size.height as _,
//         }],
//     );

//     gl.clear_attachment(
//         grr::Framebuffer::DEFAULT,
//         grr::ClearAttachment::ColorFloat(0, [0.5, 0.5, 0.5, 1.0]),
//     );

//     gl.draw(grr::Primitive::Triangles, 0..3, 0..1);

//     context.swap_buffers();
// }
