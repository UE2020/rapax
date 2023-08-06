use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use std::sync::Arc;

fn main() {
    let (gl, window, event_loop) = unsafe {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Triangle Demo")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            //.with_multisampling(16)
            .build_windowed(window_builder, &event_loop)
            .unwrap()
            .make_current()
            .unwrap();
        let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
        (Arc::new(gl), window, event_loop)
    };
    let mut ctx = rapax::ManagedContext::new(gl);
    let program = rapax::ShaderProgram::new(
        &mut ctx,
        r#"#version 330 core

layout (location = 0) in vec2 position;

out vec4 color;

void main()
{
	gl_Position = vec4(position, 0.0, 1.0);
	color = vec4((position.x + 1.0) / 2.0, (position.y + 1.0) / 2.0, 0.5, 1.0);
}
		"#,
        r#"#version 330 core

in vec4 color;

out vec4 FragColor;

void main()
{
	FragColor = color;
}
"#,
    );

    let pipeline = rapax::RenderPipeline::new(program).with_vertex_attribute(
        rapax::VertexAttributeDescriptor {
            buffer_index: 0,
            size: 2,
            ty: rapax::DataType::Float,
            normalized: false,
            stride: 0,
            offset: 0,
            divisor: 0,
        },
    );

    let vertex_data: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];
    let vertex_buffer = rapax::BufferHandle::array_buffer(
        &mut ctx,
        rapax::BufferUsage::Immutable,
        bytemuck::cast_slice(&vertex_data),
    )
    .unwrap();

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
                ctx.clear(rapax::ClearFlags::COLOR);

                ctx.with_pipeline(&pipeline, |dctx| {
                    dctx.apply_bindings(&[&vertex_buffer], None::<&rapax::BufferHandle>);
                    dctx.draw_arrays(rapax::DrawMode::Triangles, 0, 3);
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
