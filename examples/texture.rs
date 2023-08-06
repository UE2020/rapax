use glutin::event::{Event, WindowEvent};
use glutin::event_loop::ControlFlow;
use image::io::Reader as ImageReader;
use std::sync::Arc;

fn main() {
    let (gl, window, event_loop) = unsafe {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Texture Demo")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
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

		out vec4 color;
		out vec2 texcoord_out;

		void main()
		{
			gl_Position = vec4(position, 0.0, 1.0);
			color = vec4((position.x + 1.0) / 2.0, (position.y + 1.0) / 2.0, 0.5, 1.0);
			texcoord_out = texcoord;
		}
		"#,
        r#"#version 330 core

		in vec4 color;
		in vec2 texcoord_out;
		out vec4 FragColor;
		uniform sampler2D uTexture;

		void main()
		{
			FragColor = texture(uTexture, texcoord_out);
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
		0.5,  0.5,  1.0, 1.0,   // top right
    	 0.5, -0.5, 1.0, 0.0,   // bottom right
    	-0.5, -0.5, 0.0, 0.0,   // bottom let
    	-0.5,  0.5, 0.0, 1.0    // top let 
	];
    let vertex_buffer = rapax::BufferHandle::array_buffer(
        &mut ctx,
        rapax::BufferUsage::Immutable,
        bytemuck::cast_slice(&vertex_data),
    )
    .unwrap();

    let img = ImageReader::open("./examples/0001.jpg")
        .unwrap()
        .decode()
        .unwrap();
    let converted = img.into_rgb8();
    let texture = rapax::texture::TextureHandle::new(
        &mut ctx,
        rapax::texture::TextureWrap::ClampToBorder,
        rapax::texture::TextureWrap::ClampToBorder,
        rapax::texture::TextureFilteringMode::Linear,
        rapax::texture::TextureFilteringMode::Linear,
    )
    .expect("failed to create texture");

    let texture = texture.allocate_2d_data(
        &mut ctx,
        Some(converted.as_raw()),
		rapax::texture::InternalTextureFormat::Rgb,
        rapax::texture::TextureFormat::Rgb,
        converted.width() as _,
        converted.height() as _,
        rapax::DataType::UnsignedByte,
    );

	// test subimage
	let data = [0u8; 100 * 100 * 3];
	texture.write_subimage(&mut ctx, 500, 100, 100, 100, rapax::texture::TextureFormat::Rgb, rapax::DataType::UnsignedByte, &data);

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
                    dctx.apply_textures(&[(&texture, "uTexture")]);
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
