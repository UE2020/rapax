use std::sync::Arc;

use glow::HasContext;

use cgmath::*;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

fn main() {
    let (gl, window, event_loop) = unsafe {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Spiral Demo")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4)
            .build_windowed(window_builder, &event_loop)
            .unwrap()
            .make_current()
            .unwrap();
        let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
        (Arc::new(gl), window, event_loop)
    };

    let max_samples = unsafe { gl.get_parameter_i32(glow::MAX_SAMPLES) };
    println!("Max samples detected: {}", max_samples);
    let mut ctx = rapax::ManagedContext::new(gl);
    let program = rapax::ShaderProgram::new(
        &mut ctx,
        r#"#version 410
        const vec2 verts[3] = vec2[3](
            vec2(100.0f, 200.0f),
            vec2(0.0f, 0.0f),
            vec2(200.0f, 0.0f)
        );
        out vec2 vert;
        uniform mat4 uMVP;
        void main() {
            vert = verts[gl_VertexID];
            gl_Position = uMVP * vec4(vert + vec2(gl_InstanceID * 50.0), 0.0, 1.0);
        }"#,
        r#"#version 410
        precision mediump float;
        in vec2 vert;
        out vec4 color;

        uniform vec4 uColor;
        void main() {
            color = uColor;
        }"#,
    );

    let pipeline = rapax::RenderPipeline::new(program);

    let mut rotation = 0.0f32;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                let size = window.window().inner_size();

                rotation += 0.01;

                ctx.clear(rapax::ClearFlags::COLOR);

                ctx.with_pipeline(&pipeline, |dctx| {
                    dctx.set_uniform_float4("uColor", &[1.0, 0.0, 0.2, 1.0]);

                    let view = ortho(0.0, size.width as f32, size.height as f32, 0.0, 0.0, 1.0);
                    let model: Matrix4<f32> = Matrix4::from_translation(vec3(500.0, 500.0, 0.0))
                        * Matrix4::from_angle_z(Rad(rotation));
                    let mvp = view * model;
                    dctx.set_uniform_mat4("uMVP", &mvp.as_ref(), false);

                    dctx.draw_arrays_instanced(rapax::DrawMode::Triangles, 0, 3, 10);
                });
                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                    ctx.set_viewport(
                        0,
                        0,
                        physical_size.width as i32,
                        physical_size.height as i32,
                    )
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    });
}
