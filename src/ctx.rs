use super::*;

use std::sync::Arc;

/// The primitive mode used when calling draw\* functions.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum DrawMode {
    /// Draws a triangle for a group of three vertices
    Triangles = 0x4,
    /// Draws a line between a pair of vertices
    Lines = 0x1,
    /// Draws a single dot
    Points = 0x0,
}

impl DrawMode {
    pub fn to_gl(&self) -> u32 {
        *self as u32
    }
}

/// Context state
#[derive(Debug, Default, Clone)]
struct ContextState {
    // bound objects
    bound_array_buffer: Option<NativeBuffer>,
    bound_element_buffer: Option<NativeBuffer>,
    bound_textures: [Option<NativeTexture>; 16],

    // blend state
    blend_enabled: bool,
    blend_func: (u32, u32),

    // depth test
    depth_enabled: bool,

    // depth write & color write
    depth_write: bool,
    color_write: bool,

    // currently used program
    program: Option<NativeProgram>,
}

impl ContextState {
    pub fn new() -> Self {
        Self {
            depth_write: true,
            color_write: true,
            ..Default::default()
        }
    }
}

/// OpenGL context state manager.
#[derive(Debug)]
pub struct ManagedContext {
    pub(crate) gl: Arc<glow::Context>,
    last_flushed_state: ContextState,
    current_state: ContextState,
    state_stack: Vec<ContextState>,
}

impl ManagedContext {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl,
            last_flushed_state: ContextState::default(),
            current_state: ContextState::default(),
            state_stack: Vec::new(),
        }
    }

    /// Store the current state on the stack.
    pub fn save(&mut self) {
        self.state_stack.push(self.current_state.clone());
    }

    /// Restore the last saved state.
    /// ## Panics
    /// Restoring the last state on the stack will panic if the stack is empty.
    pub fn restore(&mut self) {
        self.current_state = self.state_stack.pop().unwrap();
    }

    /// Install the passed program object as a part of the current rendering state.
    pub fn use_program(&mut self, program: impl ProgramSource) {
        let program = Some(program.native_program());
        if self.last_flushed_state.program != program {
            unsafe { self.gl.use_program(program) };
            self.current_state.program = program;
            self.last_flushed_state.program = program;
        }
    }

    /// Bind a buffer object to the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn bind_array_buffer(&mut self, buffer: impl BufferSource) {
        let buffer = Some(buffer.native_buffer());
        if self.last_flushed_state.bound_array_buffer != buffer {
            unsafe { self.gl.bind_buffer(ARRAY_BUFFER, buffer) };
            self.current_state.bound_array_buffer = buffer;
            self.last_flushed_state.bound_array_buffer = buffer;
        }
    }

    /// Bind a buffer object to the index buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn bind_index_buffer(&mut self, buffer: impl BufferSource) {
        let buffer = Some(buffer.native_buffer());
        if self.last_flushed_state.bound_element_buffer != buffer {
            unsafe { self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, buffer) };
            self.current_state.bound_element_buffer = buffer;
            self.last_flushed_state.bound_element_buffer = buffer;
        }
    }

    /// Bind a buffer object. The binding point will be determined using `BufferHandle::buffer_type`.
    pub fn bind_any_buffer(&mut self, buffer: &BufferHandle) {
        let (currently_bound, last_flushed, target) = match buffer.buffer_type() {
            BufferType::ArrayBuffer => (
                &mut self.current_state.bound_array_buffer,
                &mut self.last_flushed_state.bound_array_buffer,
                ARRAY_BUFFER,
            ),
            BufferType::ElementArrayBuffer => (
                &mut self.current_state.bound_element_buffer,
                &mut self.last_flushed_state.bound_element_buffer,
                ELEMENT_ARRAY_BUFFER,
            ),
        };
        let buffer = Some(buffer.native_buffer());
        if *last_flushed != buffer {
            unsafe { self.gl.bind_buffer(target, buffer) };
            *currently_bound = buffer;
            *last_flushed = buffer;
        }
    }

    /// Render primitives using bound vertex data & index data.
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_elements(&mut self, mode: DrawMode, count: u32, ty: IndexType, offset: i32) {
        unsafe {
            self.gl
                .draw_elements(mode.to_gl(), count as i32, ty as u32, offset);
        }
    }

    pub fn invalidate_texture(&mut self, unit: u8) {
        //self.last_flushed_state.bound_textures[unit as usize] = NativeTexture(0);
    }

    /// Flush the internal cached state to the OpenGL context.
    /// ## Panics
    /// This **must** be called before calling any draw\* functions.
    pub fn flush_state(&mut self) {
        unsafe {
            if self.last_flushed_state.bound_array_buffer != self.current_state.bound_array_buffer {
                self.gl
                    .bind_buffer(ARRAY_BUFFER, self.current_state.bound_array_buffer);
            }

            if self.last_flushed_state.bound_element_buffer
                != self.current_state.bound_element_buffer
            {
                self.gl.bind_buffer(
                    ELEMENT_ARRAY_BUFFER,
                    self.current_state.bound_element_buffer,
                );
            }

            for ((i, current_texture), (_, old_texture)) in self
                .current_state
                .bound_textures
                .iter()
                .enumerate()
                .zip(self.last_flushed_state.bound_textures.iter().enumerate())
            {
                if current_texture != old_texture {
                    self.gl.active_texture(TEXTURE0 + i as u32);
                    self.gl.bind_texture(TEXTURE_2D, *current_texture);
                }
            }

            if self.last_flushed_state.blend_enabled && !self.current_state.blend_enabled {
                self.gl.disable(BLEND);
            } else if !self.last_flushed_state.blend_enabled && self.current_state.blend_enabled {
                self.gl.enable(BLEND);
            }

            if self.last_flushed_state.blend_func != self.current_state.blend_func {
                self.gl.blend_func(
                    self.current_state.blend_func.0,
                    self.current_state.blend_func.1,
                );
            }

            if self.last_flushed_state.depth_enabled && !self.current_state.depth_enabled {
                self.gl.disable(DEPTH_TEST);
            } else if !self.last_flushed_state.depth_enabled && self.current_state.depth_enabled {
                self.gl.enable(DEPTH_TEST);
            }

            if self.last_flushed_state.color_write != self.current_state.color_write {
                self.gl.color_mask(
                    self.current_state.color_write,
                    self.current_state.color_write,
                    self.current_state.color_write,
                    self.current_state.color_write,
                );
            }

            if self.last_flushed_state.depth_write != self.current_state.depth_write {
                self.gl.depth_mask(self.current_state.depth_write);
            }

            if self.last_flushed_state.program != self.current_state.program {
                self.gl.use_program(self.current_state.program);
            }
        }

        self.last_flushed_state = self.current_state.clone();
    }
}
