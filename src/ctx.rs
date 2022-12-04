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
    current_program: Option<Arc<ShaderProgram>>,
    default_vao: NativeVertexArray
}

impl ManagedContext {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl: gl.clone(),
            current_program: None,
            default_vao: unsafe { gl.create_vertex_array().expect("vertex array is required") }
        }
    }

    /// Apply a rendering pipline to the context. There must be a pipeline applied in order for the `set_uniform_*` series of functions to work.
    pub fn set_pipeline(&mut self, pipeline: &RenderPipeline) {        
        unsafe {
            if pipeline.blend_enabled {
                self.gl.enable(BLEND);
                self.gl
                    .blend_func(pipeline.blend_func.0, pipeline.blend_func.1);
            } else {
                self.gl.disable(BLEND);
            }

            if pipeline.depth_enabled {
                self.gl.enable(DEPTH_TEST);
            } else {
                self.gl.disable(DEPTH_TEST);
            }

            self.gl.color_mask(
                pipeline.color_write[0],
                pipeline.color_write[1],
                pipeline.color_write[2],
                pipeline.color_write[3],
            );

            self.gl.depth_mask(pipeline.depth_write);

            self.gl.use_program(Some(pipeline.program.program));

            self.gl.bind_vertex_array(Some(self.default_vao));
            // setup attributes
            for (index, attribute) in pipeline.vertex_attributes.iter().enumerate() {
                self.gl.vertex_attrib_pointer_f32(
                    index as _,
                    attribute.size,
                    attribute.data_type as _,
                    attribute.normalized,
                    attribute.stride,
                    attribute.offset,
                );

                self.gl.vertex_attrib_divisor(index as u32, attribute.divisor);

                self.gl.enable_vertex_attrib_array(index as _);
            }
        }

        self.current_program = Some(pipeline.program.clone());
    }

    /// Set a float4 uniform on the currently applied pipeline.
    pub fn set_uniform_float4(&self, name: &str, value: &[f32; 4]) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;
            let loc = self.gl.get_uniform_location(program, name);
            self.gl
                .uniform_4_f32(loc.as_ref(), value[0], value[1], value[2], value[3]);
        }
    }

    /// Set a float3 uniform on the currently applied pipeline.
    pub fn set_uniform_float3(&self, name: &str, value: &[f32; 3]) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;

            let loc = self.gl.get_uniform_location(program, name);
            self.gl
                .uniform_3_f32(loc.as_ref(), value[0], value[1], value[2]);
        }
    }

    /// Set a float3 uniform on the currently applied pipeline.
    pub fn set_uniform_float2(&self, name: &str, value: &[f32; 2]) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;

            let loc = self.gl.get_uniform_location(program, name);
            self.gl.uniform_2_f32(loc.as_ref(), value[0], value[1]);
        }
    }

    /// Set a float1 uniform on the currently applied pipeline.
    pub fn set_uniform_float1(&self, name: &str, value: f32) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;

            let loc = self.gl.get_uniform_location(program, name);
            self.gl.uniform_1_f32(loc.as_ref(), value);
        }
    }

    /// Set a mat2 uniform on the currently applied pipeline.
    /// If you're not sure what `transpose` means, simply make it false.
    pub fn set_uniform_mat2(&self, name: &str, value: &[f32; 4], transpose: bool) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;

            let loc = self.gl.get_uniform_location(program, name);

            self.gl
                .uniform_matrix_2_f32_slice(loc.as_ref(), transpose, value);
        }
    }

    /// Set a mat3 uniform on the currently applied pipeline.
    /// If you're not sure what `transpose` means, simply make it false.
    pub fn set_uniform_mat3(&self, name: &str, value: &[f32; 9], transpose: bool) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;

            let loc = self.gl.get_uniform_location(program, name);

            self.gl
                .uniform_matrix_3_f32_slice(loc.as_ref(), transpose, value);
        }
    }

    /// Set a mat4 uniform on the currently applied pipeline.
    /// If you're not sure what `transpose` means, simply make it false.
    pub fn set_uniform_mat4(&self, name: &str, value: &[f32; 16], transpose: bool) {
        unsafe {
            let program = self.current_program.as_ref().expect("there should be a bound pipeline").program;

            let loc = self.gl.get_uniform_location(program, name);
            self.gl
                .uniform_matrix_4_f32_slice(loc.as_ref(), transpose, value);
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

    /// Render primitives using bound vertex data & index data.
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_elements(&mut self, mode: DrawMode, count: u32, ty: DataType, offset: i32) {
        unsafe {
            self.gl
                .draw_elements(mode.to_gl(), count as i32, ty as u32, offset);
        }
    }

    /// Render primitives using bound vertex data & index data, with instancing.
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_elements_instanced(&mut self, mode: DrawMode, count: u32, ty: DataType, offset: i32, instances: u32) {
        unsafe {
            self.gl
                .draw_elements_instanced(mode.to_gl(), count as i32, ty as u32, offset, instances as _);
        }
    }

    /// Render primitives using bound vertex data.
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_arrays(&mut self, mode: DrawMode, first: i32, count: i32) {
        unsafe {
            self.gl.draw_arrays(mode.to_gl(), first as i32, count);
        }
    }

    /// Render primitives using bound vertex data, with instancing.
    /// Calling `flush_state` before calling any draw\* functions is highly advised.
    pub fn draw_arrays_instanced(&mut self, mode: DrawMode, first: i32, count: i32, instances: u32) {
        unsafe {
            self.gl.draw_arrays_instanced(mode.to_gl(), first as i32, count, instances as _);
        }
    }

    /// Clear buffers.
    /// Calling `flush_state` is highly advised before calling this function.
    pub fn clear(&mut self, mask: u32) {
        unsafe {
            self.gl.clear(mask);
        }
    }

    /// Set the clear color.
    pub fn set_clear_color(&mut self, color: [f32; 4]) {
        unsafe { self.gl.clear_color(color[0], color[1], color[2], color[3]) };
    }

    /// Set the viewport position & dimensions.
    pub fn set_viewport(&mut self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { self.gl.viewport(x, y, w, h) };
    }
}
