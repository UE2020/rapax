use std::sync::Arc;

use glow::HasContext;

use cgmath::*;

use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;

fn main() {
    let (gl, window, event_loop) = unsafe {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Instancing Demo")
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
		out float instanceID;
        uniform mat4 uMVP;
        void main() {
            vert = verts[gl_VertexID];
			instanceID = gl_InstanceID;
            gl_Position = uMVP * vec4(vert + vec2(gl_InstanceID * 100.0, 0.0), 0.0, 1.0);
        }"#,
        r#"#version 410
        precision mediump float;
        in vec2 vert;
		in float instanceID;
        out vec4 color;

        uniform vec4 uColor;
        void main() {
			vec4 instanceColor = vec4(uColor.r / (instanceID + 1.0f), uColor.gba);
            color = instanceColor;
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

                rotation += 0.1;

                ctx.clear(rapax::ClearFlags::COLOR);

                ctx.with_pipeline(&pipeline, |dctx| {
                    dctx.set_uniform_float4("uColor", &[1.0, 0.0, 0.2, 1.0]);

                    let view = ortho(0.0, size.width as f32, size.height as f32, 0.0, 0.0, 1.0);
                    let model: Matrix4<f32> = Matrix4::from_translation(vec3(
                        100.0,
                        100.0 + (rotation.sin() * 10.0),
                        0.0,
                    ));
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
