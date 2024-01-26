use clipboard::{ClipboardContext, ClipboardProvider};
use cosmic_text::fontdb::Source;
use cosmic_text::{
    Action, Attrs, Buffer, Color, Edit, Editor, FontSystem, Metrics, Shaping, SwashCache,
};
use glutin::event::{
    ElementState, Event, ModifiersState, MouseScrollDelta, VirtualKeyCode, WindowEvent,
};
use glutin::event_loop::ControlFlow;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

/// From https://github.com/pop-os/cosmic-text/blob/main/examples/editor-libcosmic/src/text_box.rs#L59C1-L107C1
fn draw_pixel(
    buffer: &mut [u8],
    width: i32,
    height: i32,
    x: i32,
    y: i32,
    color: cosmic_text::Color,
) {
    let alpha = (color.0 >> 24) & 0xFF;
    if alpha == 0 {
        // Do not draw if alpha is zero
        return;
    }

    if y < 0 || y >= height {
        // Skip if y out of bounds
        return;
    }

    if x < 0 || x >= width {
        // Skip if x out of bounds
        return;
    }

    let offset = (y as usize * width as usize + x as usize) * 4;

    let mut current = buffer[offset + 2] as u32
        | (buffer[offset + 1] as u32) << 8
        | (buffer[offset + 0] as u32) << 16
        | (buffer[offset + 3] as u32) << 24;

    if alpha >= 255 || current == 0 {
        // Alpha is 100% or current is null, replace with no blending
        current = color.0;
    } else {
        // Alpha blend with current value
        let n_alpha = 255 - alpha;
        let rb = ((n_alpha * (current & 0x00FF00FF)) + (alpha * (color.0 & 0x00FF00FF))) >> 8;
        let ag = (n_alpha * ((current & 0xFF00FF00) >> 8))
            + (alpha * (0x01000000 | ((color.0 & 0x0000FF00) >> 8)));
        current = (rb & 0x00FF00FF) | (ag & 0xFF00FF00);
    }

    buffer[offset + 2] = current as u8;
    buffer[offset + 1] = (current >> 8) as u8;
    buffer[offset + 0] = (current >> 16) as u8;
    buffer[offset + 3] = (current >> 24) as u8;
    //buffer[offset + 3] = (((color.0 >> 24) as f32 / 255.0).powf(0.8) * 255.0) as u8;
}

fn main() {
    let (gl, window, event_loop) = unsafe {
        let event_loop = glutin::event_loop::EventLoop::new();
        let window_builder = glutin::window::WindowBuilder::new()
            .with_title("Text Demo")
            .with_inner_size(glutin::dpi::LogicalSize::new(1024.0, 768.0));
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            //.with_multisampling(16)
            .with_pixel_format(24, 0)
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

        uniform mat4 view;

		void main()
		{
			gl_Position = view * vec4(position, 0.0, 1.0);
			texcoord_out = texcoord;
		}
		"#,
        r#"#version 330 core

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
        .with_blend(true)
        .with_blend_func(
            rapax::BlendFactor::SourceAlpha,
            rapax::BlendFactor::OneMinusSourceAlpha,
        )
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

    let texture = rapax::texture::TextureHandle::new(
        &mut ctx,
        rapax::texture::TextureWrap::ClampToBorder,
        rapax::texture::TextureWrap::ClampToBorder,
        rapax::texture::TextureFilteringMode::Nearest,
        rapax::texture::TextureFilteringMode::Nearest,
    )
    .expect("failed to create texture");

    let mut size = window.window().inner_size();

    let mut font_system = FontSystem::new_with_fonts([Source::File(
        PathBuf::from_str("../../Downloads/Times New Roman.ttf").unwrap(),
    )]);
    //font_system.db_mut().set_serif_family("Times New Roman");
    //dbg!(font_system.db());
    let mut swash_cache = SwashCache::new();
    let metrics = Metrics::new(16.0, 16.0 + 5.0);
    let mut buffer = Buffer::new(&mut font_system, metrics);
    buffer.set_size(&mut font_system, size.width as f32, size.height as f32);
    buffer.set_text(
        &mut font_system,
        include_str!("demo.txt"),
        Attrs::new().family(cosmic_text::Family::Name("Times New Roman")),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(&mut font_system);
    let text_color = Color::rgb(0, 0, 0);
    let mut pixels = vec![0u8; size.width as usize * size.height as usize * 4];
    buffer.draw(
        &mut font_system,
        &mut swash_cache,
        text_color,
        |x, y, w, h, color| {
            for row in 0..h as i32 {
                for col in 0..w as i32 {
                    draw_pixel(
                        &mut pixels,
                        size.width as _,
                        size.height as _,
                        x + col,
                        y + row,
                        color,
                    );
                }
            }
        },
    );

    let mut texture = texture.allocate_2d_data(
        &mut ctx,
        Some(&pixels),
        rapax::texture::InternalTextureFormat::Rgba,
        rapax::texture::TextureFormat::Rgba,
        size.width as _,
        size.height as _,
        rapax::DataType::UnsignedByte,
    );

    let mut editor = Editor::new(buffer);

    let rerender = |editor: &mut Editor,
                    font_system: &mut FontSystem,
                    texture: &mut rapax::Texture2D,
                    size: glutin::dpi::PhysicalSize<u32>,
                    swash_cache: &mut SwashCache,
                    ctx: &mut rapax::ManagedContext| {
        editor.shape_as_needed(font_system);
        let text_color = Color::rgb(0, 0, 0);
        let mut pixels = vec![0u8; size.width as usize * size.height as usize * 4];
        editor.draw(font_system, swash_cache, text_color, |x, y, w, h, color| {
            for row in 0..h as i32 {
                for col in 0..w as i32 {
                    draw_pixel(
                        &mut pixels,
                        size.width as _,
                        size.height as _,
                        x + col,
                        y + row,
                        color,
                    );
                }
            }
        });

        texture.write_subimage(
            ctx,
            0,
            0,
            size.width as _,
            size.height as _,
            rapax::texture::TextureFormat::Rgba,
            rapax::DataType::UnsignedByte,
            &pixels,
        );
    };

    let mut clicking = false;

    let mut mouse_x = 0;
    let mut mouse_y = 0;

    window
        .window()
        .set_cursor_icon(glutin::window::CursorIcon::Text);

    let mut modifiers = ModifiersState::default();

    let mut clipboard = ClipboardContext::new().unwrap();

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
                ctx.set_clear_color([1., 1., 1., 1.0]);
                ctx.clear(rapax::ClearFlags::COLOR);
                ctx.with_pipeline(&pipeline, |dctx| {
                    size = window.window().inner_size();
                    dctx.apply_bindings(&[&vertex_buffer], Some(&index_buffer));
                    dctx.apply_textures(&[(&texture, "uTexture")]);
                    dctx.set_uniform_mat4(
                        "view",
                        (cgmath::ortho(0.0, size.width as f32, size.height as f32, 0.0, 0.0, 1.0)
                            * cgmath::Matrix4::from_nonuniform_scale(
                                size.width as f32,
                                size.height as f32,
                                1.0,
                            ))
                        .as_ref(),
                        false,
                    );
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
                    );
                    size = *physical_size;
                    editor.buffer_mut().set_size(
                        &mut font_system,
                        physical_size.width as f32,
                        physical_size.height as f32,
                    );
                    editor.shape_as_needed(&mut font_system);
                    let text_color = Color::rgb(0, 0, 0);
                    let mut pixels = vec![0u8; size.width as usize * size.height as usize * 4];
                    editor.draw(
                        &mut font_system,
                        &mut swash_cache,
                        text_color,
                        |x, y, w, h, color| {
                            for row in 0..h as i32 {
                                for col in 0..w as i32 {
                                    draw_pixel(
                                        &mut pixels,
                                        size.width as _,
                                        size.height as _,
                                        x + col,
                                        y + row,
                                        color,
                                    );
                                }
                            }
                        },
                    );

                    texture.reallocate_2d_data(
                        &mut ctx,
                        Some(&pixels),
                        rapax::texture::InternalTextureFormat::Rgba,
                        rapax::texture::TextureFormat::Rgba,
                        physical_size.width as _,
                        physical_size.height as _,
                        rapax::DataType::UnsignedByte,
                    );
                }
                WindowEvent::KeyboardInput {
                    input,
                    ..
                } => {
                    if let Some(keycode) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            match keycode {
                                VirtualKeyCode::Left => {
                                    editor.action(&mut font_system, Action::Left)
                                }
                                VirtualKeyCode::Right => {
                                    editor.action(&mut font_system, Action::Right)
                                }
                                VirtualKeyCode::Up => editor.action(&mut font_system, Action::Up),
                                VirtualKeyCode::Down => {
                                    editor.action(&mut font_system, Action::Down)
                                }
                                VirtualKeyCode::Escape => {
                                    editor.action(&mut font_system, Action::Escape)
                                }
                                VirtualKeyCode::Return => {
                                    editor.action(&mut font_system, Action::Enter)
                                }
                                VirtualKeyCode::Back => {
                                    editor.action(&mut font_system, Action::Backspace)
                                }
                                VirtualKeyCode::Delete => {
                                    editor.action(&mut font_system, Action::Delete)
                                }
                                VirtualKeyCode::C => {
                                    if modifiers.ctrl() {
                                        if let Some(content) = editor.copy_selection() {
                                            clipboard.set_contents(content).unwrap();
                                        }
                                    }
                                }
                                VirtualKeyCode::X => {
                                    if modifiers.ctrl() {
                                        if let Some(content) = editor.copy_selection() {
                                            clipboard.set_contents(content).unwrap();
                                            editor.delete_selection();
                                        }
                                    }
                                }
                                VirtualKeyCode::V => {
                                    if modifiers.ctrl() {
                                        editor.insert_string(
                                            &clipboard.get_contents().unwrap(),
                                            None,
                                        );
                                    }
                                }
                                VirtualKeyCode::A => {
                                    if modifiers.ctrl() {
                                        editor.insert_string(
                                            &clipboard.get_contents().unwrap(),
                                            None,
                                        );
                                    }
                                }
                                _ => {}
                            }

                            rerender(
                                &mut editor,
                                &mut font_system,
                                &mut texture,
                                size,
                                &mut swash_cache,
                                &mut ctx,
                            );
                        }
                    }
                }
                WindowEvent::MouseWheel {
                    delta,
                    ..
                } => {
                    editor.action(
                        &mut font_system,
                        Action::Scroll {
                            lines: match delta {
                                MouseScrollDelta::LineDelta(_, y) => {
                                    -(if y.abs() < 1.0 { 1.0 * y.signum() } else { *y }) as i32
                                }
                                MouseScrollDelta::PixelDelta(pos) => (pos.y / 21.0).ceil() as i32,
                            },
                        },
                    );
                    rerender(
                        &mut editor,
                        &mut font_system,
                        &mut texture,
                        size,
                        &mut swash_cache,
                        &mut ctx,
                    );
                }
                WindowEvent::ModifiersChanged(new_modifiers) => {
                    modifiers = *new_modifiers;
                }
                WindowEvent::ReceivedCharacter(c) => {
                    editor.action(&mut font_system, Action::Insert(*c));
                    rerender(
                        &mut editor,
                        &mut font_system,
                        &mut texture,
                        size,
                        &mut swash_cache,
                        &mut ctx,
                    );
                }
                WindowEvent::CursorMoved {
                    position,
                    ..
                } => {
                    mouse_x = position.x as i32;
                    mouse_y = position.y as i32;
                    if clicking {
                        editor.action(
                            &mut font_system,
                            Action::Drag {
                                x: position.x as i32,
                                y: position.y as i32,
                            },
                        );
                        rerender(
                            &mut editor,
                            &mut font_system,
                            &mut texture,
                            size,
                            &mut swash_cache,
                            &mut ctx,
                        );
                    }
                }
                WindowEvent::MouseInput {
                    state,
                    ..
                } => {
                    if *state == ElementState::Pressed {
                        editor.action(
                            &mut font_system,
                            Action::Click {
                                x: mouse_x,
                                y: mouse_y,
                            },
                        );
                        rerender(
                            &mut editor,
                            &mut font_system,
                            &mut texture,
                            size,
                            &mut swash_cache,
                            &mut ctx,
                        );
                        clicking = true;
                    } else if *state == ElementState::Released {
                        clicking = false;
                    }
                }
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }
    });
}
