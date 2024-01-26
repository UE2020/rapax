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
		layout (location = 1) in vec2 texcoord;

		out vec2 texcoord_out;

		void main()
		{
			gl_Position = vec4(position, 0.0, 1.0);
			texcoord_out = texcoord;
		}
		"#,
        r#"#version 330 core

		in vec2 texcoord_out;
		out vec4 FragColor;
		uniform sampler2D uTexture;

		void main()
		{
			//FragColor = texture(uTexture, texcoord_out);
            FragColor = vec4(1.0, 1.0, 1.0, 1.0);
		}
"#,
    );

    let pipeline = rapax::RenderPipeline::new(program)
        .with_vertex_attribute(rapax::VertexAttributeDescriptor {
            buffer_index: 0,
            size: 2,
            ty: rapax::DataType::Float,
            normalized: false,
            stride: 4 * rapax::DataType::Float.sizeof() as i32,
            offset: 0,
            divisor: 0,
        })
        .with_vertex_attribute(rapax::VertexAttributeDescriptor {
            buffer_index: 0,
            size: 2,
            ty: rapax::DataType::Float,
            normalized: false,
            stride: 4 * rapax::DataType::Float.sizeof() as i32,
            offset: 2 * rapax::DataType::Float.sizeof() as i32,
            divisor: 0,
        });

    #[rustfmt::skip]
    let vertex_data: [f32; 16] = [
        0.0,  0.0,  0.0, 0.0,   // top left
        1.0, 0.0, 1.0, 0.0,   // top right
        0.0, 1.0, 0.0, 1.0,   // bottom left
        1.0,  1.0, 1.0, 1.0    // bottom right
    ];

    #[rustfmt::skip]
    let index_data: [u16; 6] = [
        0, 1, 2,
        2, 3, 1
    ];

    let vertex_buffer = rapax::BufferHandle::array_buffer(
        &mut ctx,
        rapax::BufferUsage::Immutable,
        bytemuck::cast_slice(&vertex_data),
    )
    .unwrap();

    let index_buffer = rapax::BufferHandle::index_buffer(
        &mut ctx,
        rapax::BufferUsage::Immutable,
        bytemuck::cast_slice(&index_data),
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
                    dctx.apply_bindings(&[&vertex_buffer], Some(&index_buffer));
                    //dctx.apply_textures(&[(&texture, "uTexture")]);
                    dctx.draw_elements(
                        rapax::DrawMode::Triangles,
                        index_data.len() as _,
                        rapax::DataType::UnsignedShort,
                        0,
                    );
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
