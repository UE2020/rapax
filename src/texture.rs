use super::*;
use std::sync::Arc;

/// A handle to an OpenGL texture object. The internal OpenGL program object will be automatically freed on drop.
#[derive(Debug)]
pub struct TextureHandle {
    pub(crate) texture: NativeTexture,
    gl: Arc<Context>,
}

impl TextureHandle {
    /// Create a new texture.
    pub fn new(
        ctx: &ManagedContext,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
    ) -> Self {
        let shader = compile_shader(&ctx.gl, vertex_shader_source, fragment_shader_source);
        Self {
            program: shader,
            gl: ctx.gl.clone(),
        }
    }
}

impl BindableTexture for &TextureHandle {
    unsafe fn bind(&self, target: u32) {
		self.gl.bind_texture(target, Some(self.texture));
	}
}

// impl BindableTexture for NativeProgram {
//     fn native_program(&self) -> NativeProgram {
//         *self
//     }
// }

pub trait BindableTexture {
    unsafe fn bind(&self, target: u32);
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}