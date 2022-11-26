use crate::blend::BlendFactor;

use super::*;

use std::{default, sync::Arc};

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

#[derive(Debug, Clone, PartialEq, Eq)]
enum MaybeKnown<T: Default> {
    Known(T),
    Unknown,
}

impl<T: Default> MaybeKnown<T> {
    fn assume_known(&self) -> &T {
        match self {
            Self::Known(v) => v,
            Self::Unknown => panic!("Tried to unwrap invalidated binding"),
        }
    }
}

impl<T: Default> Default for MaybeKnown<T> {
    fn default() -> Self {
        Self::Known(T::default())
    }
}

/// Context state
#[derive(Debug, Default, Clone)]
struct ContextState {
    // bound objects
    bound_array_buffer: MaybeKnown<Option<NativeBuffer>>,
    bound_element_buffer: MaybeKnown<Option<NativeBuffer>>,
    bound_textures: [MaybeKnown<Option<NativeTexture>>; 16],

    // blend state
    blend_enabled: bool,
    blend_func: (u32, u32),

    // clear color
    clear_color: [f32; 4],

    // depth test
    depth_enabled: bool,

    // depth write & color write
    depth_write: bool,
    color_write: [bool; 4],

    // currently used program
    program: MaybeKnown<Option<NativeProgram>>,

    // viewport
    viewport: [i32; 4],
}

impl ContextState {
    pub fn new() -> Self {
        Self {
            depth_write: true,
            color_write: [true, true, true, true],
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

    /// Bind the given vertex array to the context. The vertex array will be unbound after the closure is executed to prevent state contamination.
    pub fn bind_vertex_array_with<F>(&mut self, va: impl VertexArraySource, func: F)
        where F: FnOnce(&mut ManagedContext)
    {
        // bind the vertex array
        unsafe { self.gl.bind_vertex_array(Some(va.native_vertex_array())) };
        func(self);
        unsafe { self.gl.bind_vertex_array(None) };
    }

    /// Set the blend func.
    pub fn set_blend_func(&mut self, src: BlendFactor, dst: BlendFactor) {
        if self.last_flushed_state.blend_func != (src as u32, dst as u32) {
            self.last_flushed_state.blend_func = (src as u32, dst as u32);
            self.current_state.blend_func = (src as u32, dst as u32);
            unsafe { self.gl.blend_func(src as u32, dst as u32); };
        }
    }

    /// Enable blending.
    pub fn set_blend(&mut self, enabled: bool) {
        if self.last_flushed_state.blend_enabled != enabled {
            self.last_flushed_state.blend_enabled = enabled;
            self.current_state.blend_enabled = enabled;
            match enabled {
                true => unsafe { self.gl.enable(BLEND) },
                false => unsafe { self.gl.disable(BLEND) }
            }
        }
    }

    /// Set depth write.
    pub fn set_depth_write(&mut self, enabled: bool) {
        if self.last_flushed_state.depth_write != enabled {
            self.last_flushed_state.depth_write = enabled;
            self.current_state.depth_write = enabled;
            unsafe { self.gl.depth_mask(enabled) }
        }
    }

    /// Set color write.
    pub fn set_color_write(&mut self, r: bool, g: bool, b: bool, a: bool) {
        let enabled = [r, g, b, a];
        if self.last_flushed_state.color_write != enabled {
            self.last_flushed_state.color_write = enabled;
            self.current_state.color_write = enabled;
            unsafe { self.gl.color_mask(r, g, b, a) }
        }
    }

    /// Install the passed program object as a part of the current rendering state.
    pub fn use_program(&mut self, program: impl ProgramSource) {
        let program = Some(program.native_program());
        if self.last_flushed_state.program != MaybeKnown::Known(program) {
            unsafe { self.gl.use_program(program) };
            self.current_state.program = MaybeKnown::Known(program);
            self.last_flushed_state.program = MaybeKnown::Known(program);
        }
    }

    /// Uninstall the current shader program as a part of the current rendering state.
    pub fn unuse_program(&mut self) {
        if self.last_flushed_state.program != MaybeKnown::Known(None) {
            unsafe { self.gl.use_program(None) };
            self.current_state.program = MaybeKnown::Known(None);
            self.last_flushed_state.program = MaybeKnown::Known(None);
        }
    }

    /// Bind a buffer object to the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn bind_array_buffer(&mut self, buffer: impl BufferSource) {
        let buffer = Some(buffer.native_buffer());
        if self.last_flushed_state.bound_array_buffer != MaybeKnown::Known(buffer) {
            unsafe { self.gl.bind_buffer(ARRAY_BUFFER, buffer) };
            self.current_state.bound_array_buffer = MaybeKnown::Known(buffer);
            self.last_flushed_state.bound_array_buffer = MaybeKnown::Known(buffer);
        }
    }

    /// Unbind the currently bound buffer object from the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn unbind_array_buffer(&mut self) {
        if self.last_flushed_state.bound_array_buffer != MaybeKnown::Known(None) {
            unsafe { self.gl.bind_buffer(ARRAY_BUFFER, None) };
            self.current_state.bound_array_buffer = MaybeKnown::Known(None);
            self.last_flushed_state.bound_array_buffer = MaybeKnown::Known(None);
        }
    }

    /// Bind a buffer object to the index buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn bind_index_buffer(&mut self, buffer: impl BufferSource) {
        let buffer = Some(buffer.native_buffer());
        if self.last_flushed_state.bound_element_buffer != MaybeKnown::Known(buffer) {
            unsafe { self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, buffer) };
            self.current_state.bound_element_buffer = MaybeKnown::Known(buffer);
            self.last_flushed_state.bound_element_buffer = MaybeKnown::Known(buffer);
        }
    }

    /// Unbind the currently bound buffer object from the array buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn unbind_index_buffer(&mut self) {
        if self.last_flushed_state.bound_element_buffer != MaybeKnown::Known(None) {
            unsafe { self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, None) };
            self.current_state.bound_element_buffer = MaybeKnown::Known(None);
            self.last_flushed_state.bound_element_buffer = MaybeKnown::Known(None);
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
        if *last_flushed != MaybeKnown::Known(buffer) {
            unsafe { self.gl.bind_buffer(target, buffer) };
            *currently_bound = MaybeKnown::Known(buffer);
            *last_flushed = MaybeKnown::Known(buffer);
        }
    }

    /// Render primitives using bound vertex data & index data.
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_elements(&mut self, mode: DrawMode, count: u32, ty: DataType, offset: i32) {
        unsafe {
            self.gl
                .draw_elements(mode.to_gl(), count as i32, ty as u32, offset);
        }
    }

    /// Render primitives using bound vertex data
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_arrays(&mut self, mode: DrawMode, first: i32, count: i32) {
        unsafe {
            self.gl.draw_arrays(mode.to_gl(), first as i32, count);
        }
    }

    /// Clear buffers
    /// Calling `flush_state` is highly advised before calling this function.
    pub fn clear(&mut self, mask: u32) {
        unsafe {
            self.gl.clear(mask);
        }
    }

    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        if self.last_flushed_state.clear_color != color {
            unsafe { self.gl.clear_color(color[0], color[1], color[2], color[3]) };
            self.current_state.clear_color = color;
            self.last_flushed_state.clear_color = color;
        }
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        let vp = [x, y, w, h];
        if self.last_flushed_state.viewport != vp {
            unsafe { self.gl.viewport(x, y, w, h) };
            self.current_state.viewport = vp;
            self.last_flushed_state.viewport = vp;
        }
    }

    /// Mark a texture in the cache as invalid. This is required after making a foreign binding.
    /// ## Example
    /// ```rust
    /// ctx.flush_state(); // more on this function later
    /// gl.active_texture(glow::TEXTURE0);
    /// glXBindTexImageEXT(...);
    /// ctx.invalidate_texture(0); // the input is the currently selected texture unit, this must be done when you make a foreign binding call
    /// ctx.draw_elements(...);
    /// glXReleaseTexImageEXT(...);
    /// ```
    pub fn invalidate_texture(&mut self, unit: u8) {
        self.last_flushed_state.bound_textures[unit as usize] = MaybeKnown::Unknown;
    }

    /// Mark the cached index buffer as invalid.
    pub fn invalidate_index_buffer(&mut self) {
        self.last_flushed_state.bound_element_buffer = MaybeKnown::Unknown;
    }

    /// Mark the cached array buffer as invalid.
    pub fn invalidate_array_buffer(&mut self) {
        self.last_flushed_state.bound_array_buffer = MaybeKnown::Unknown;
    }

    /// Flush the internal cached state to the OpenGL context.
    /// ## Panics
    /// This **must** be called before calling any draw\* functions.
    pub fn flush_state(&mut self) {
        unsafe {
            if self.last_flushed_state.bound_array_buffer != self.current_state.bound_array_buffer {
                self.gl.bind_buffer(
                    ARRAY_BUFFER,
                    *self.current_state.bound_array_buffer.assume_known(),
                );
            }

            if self.last_flushed_state.bound_element_buffer
                != self.current_state.bound_element_buffer
            {
                self.gl.bind_buffer(
                    ELEMENT_ARRAY_BUFFER,
                    *self.current_state.bound_element_buffer.assume_known(),
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
                    self.gl
                        .bind_texture(TEXTURE_2D, *current_texture.assume_known());
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

            if self.last_flushed_state.clear_color != self.current_state.clear_color {
                self.gl.clear_color(
                    self.current_state.clear_color[0],
                    self.current_state.clear_color[1],
                    self.current_state.clear_color[2],
                    self.current_state.clear_color[3],
                );
            }

            if self.last_flushed_state.depth_enabled && !self.current_state.depth_enabled {
                self.gl.disable(DEPTH_TEST);
            } else if !self.last_flushed_state.depth_enabled && self.current_state.depth_enabled {
                self.gl.enable(DEPTH_TEST);
            }

            if self.last_flushed_state.color_write != self.current_state.color_write {
                self.gl.color_mask(
                    self.current_state.color_write[0],
                    self.current_state.color_write[1],
                    self.current_state.color_write[2],
                    self.current_state.color_write[3],
                );
            }

            if self.last_flushed_state.depth_write != self.current_state.depth_write {
                self.gl.depth_mask(self.current_state.depth_write);
            }

            if self.last_flushed_state.program != self.current_state.program {
                self.gl
                    .use_program(*self.current_state.program.assume_known());
            }

            if self.last_flushed_state.viewport != self.current_state.viewport {
                self.gl.viewport(
                    self.current_state.viewport[0],
                    self.current_state.viewport[1],
                    self.current_state.viewport[2],
                    self.current_state.viewport[3],
                );
            }
        }

        self.last_flushed_state = self.current_state.clone();
    }
}
