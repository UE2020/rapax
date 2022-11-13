use std::sync::Arc;

fn main() {
    let (gl, window, event_loop) = {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Hello triangle!")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let window = unsafe {
            glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(window_builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };
        let gl = unsafe {
            glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _)
        };
        (Arc::new(gl), window, event_loop)
    };

    let mut ctx = rapax::ManagedContext::new(gl);
    let va = rapax::VertexArrayObject::new(&mut ctx);
    ctx.bind_vertex_array(&va);
    let program = rapax::ShaderProgram::new(
        &mut ctx,
        r#"#version 410
        const vec2 verts[3] = vec2[3](
            vec2(0.5f, 1.0f),
            vec2(0.0f, 0.0f),
            vec2(1.0f, 0.0f)
        );
        out vec2 vert;
        void main() {
            vert = verts[gl_VertexID];
            gl_Position = vec4(vert - 0.5, 0.0, 1.0);
        }"#,
        r#"#version 410
        precision mediump float;
        in vec2 vert;
        out vec4 color;
        void main() {
            color = vec4(vert, 0.5, 1.0);
        }"#,
    );

    ctx.use_program(&program);

    use glutin::event::{Event, WindowEvent};
    use glutin::event_loop::ControlFlow;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                //gl.clear(glow::COLOR_BUFFER_BIT);
                ctx.flush_state();
                ctx.clear(rapax::COLOR_BUFFER_BIT);
                ctx.draw_arrays(rapax::DrawMode::Triangles, 0, 3);
                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                    ctx.set_viewport(0, 0, physical_size.width as i32, physical_size.height as i32)
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    });
}
