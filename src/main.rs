#![feature(raw_ref_op, const_fn_trait_bound)]

use winit::{dpi::LogicalSize, event_loop::*};

mod collections;
mod opengl;
mod pos;
mod render;

extern crate gl;
extern crate log;

const DEFAULT_VERTEX_SRC: &str = r#"
    #version 450 core

    layout (location = 0) in vec2 v_pos;
    layout (location = 1) in vec3 v_color;

    layout (location = 0) out vec3 a_color;

    void main() {
        a_color = v_color;
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

const VERTICES: [f32; 15] = [
    -0.5, -0.5, 1.0, 0.0, 0.0, 0.5, -0.5, 0.0, 1.0, 0.0, 0.0, 0.5, 0.0, 0.0, 1.0,
];

fn main() {
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

    use opengl::shader::{Fragment, ShaderProgram, Vertex};
    let vertex_shader = ShaderProgram::<Vertex>::new(&[DEFAULT_VERTEX_SRC]);
    let vertex_shader = ShaderProgram::<Fragment>::new(&[DEFAULT_FRAGMENT_SRC]);
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
