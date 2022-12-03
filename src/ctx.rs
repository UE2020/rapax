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

/// OpenGL context state manager.
#[derive(Debug)]
pub struct ManagedContext {
    pub(crate) gl: Arc<glow::Context>,
}

impl ManagedContext {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl,
        }
    }

    pub fn set_pipeline(&self, pipeline: &RenderPipeline) {
        // Program must be reset under ALL CIRCUMSTANCES
        unsafe {
            if pipeline.blend_enabled {
                self.gl.enable(BLEND);
            } else {
                self.gl.disable(BLEND);
            }

            self.gl
                .blend_func(pipeline.blend_func.0, pipeline.blend_func.1);

            if pipeline.depth_enabled {
                self.gl.disable(DEPTH_TEST);
            } else {
                self.gl.enable(DEPTH_TEST);
            }

            self.gl.color_mask(
                pipeline.color_write[0],
                pipeline.color_write[1],
                pipeline.color_write[2],
                pipeline.color_write[3],
            );

            self.gl.depth_mask(pipeline.depth_write);
        }
    }

    /// Bind a buffer object to the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn bind_array_buffer(&mut self, buffer: impl BufferSource) {
        let buffer = Some(buffer.native_buffer());
        unsafe { self.gl.bind_buffer(ARRAY_BUFFER, buffer) };
    }

    /// Unbind the currently bound buffer object from the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn unbind_array_buffer(&mut self) {
        unsafe { self.gl.bind_buffer(ARRAY_BUFFER, None) };
    }

    /// Bind a buffer object to the index buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn bind_index_buffer(&mut self, buffer: impl BufferSource) {
        let buffer = Some(buffer.native_buffer());
        unsafe { self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, buffer) };
    }

    /// Unbind the currently bound buffer object from the array buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn unbind_index_buffer(&mut self) {
        unsafe { self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, None) };
    }

    /// Bind a buffer object. The binding point will be determined using `BufferHandle::buffer_type`.
    pub fn bind_any_buffer(&mut self, buffer: &BufferHandle) {
        let target = match buffer.buffer_type() {
            BufferType::ArrayBuffer => ARRAY_BUFFER,
            BufferType::ElementArrayBuffer => ELEMENT_ARRAY_BUFFER,
        };
        let buffer = Some(buffer.native_buffer());
        unsafe { self.gl.bind_buffer(target, buffer) };
    }

    /// Bind the given vertex array to the context. The vertex array will be unbound after the closure is executed to prevent state contamination.
    pub fn bind_vertex_array_with<F>(&mut self, va: impl VertexArraySource, func: F)
    where
        F: FnOnce(&mut ManagedContext),
    {
        // bind the vertex array
        unsafe { self.gl.bind_vertex_array(Some(va.native_vertex_array())) };
        func(self);
        unsafe { self.gl.bind_vertex_array(None) };
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
        unsafe { self.gl.clear_color(color[0], color[1], color[2], color[3]) };
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { self.gl.viewport(x, y, w, h) };
    }
}
