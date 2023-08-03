use super::*;

use std::sync::Arc;

/// The primitive mode used when calling draw\* functions.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum DrawMode {
    /// Draws a triangle for a group of three vertices
    Triangles = TRIANGLES,
    /// Draws a line between a pair of vertices
    Lines = LINES,
    /// Draws a single dot
    Points = POINTS,
    /// Draws a strip of triangles. Every group of 3 adjacent vertices forms a triangle.
    TriangleStrip = TRIANGLE_STRIP,
	TriangleFan = TRIANGLE_FAN,
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
    default_vao: NativeVertexArray,
}

impl ManagedContext {
    pub fn new(gl: Arc<glow::Context>) -> Self {
        Self {
            gl: gl.clone(),
            default_vao: unsafe { gl.create_vertex_array().expect("vertex array is required") },
        }
    }

    /// Create a scope in which the referenced pipeline is active.
    pub fn with_pipeline(&mut self, pipeline: &RenderPipeline, draw_cb: impl FnOnce(Drawable)) {
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

            if pipeline.scissor_enabled {
                self.gl.enable(SCISSOR_TEST);
            } else {
                self.gl.disable(SCISSOR_TEST);
            }

            match &pipeline.stencil_state {
                Some(stencil) => {
                    self.gl.enable(STENCIL_TEST);
                    self.gl.stencil_mask_separate(FRONT, stencil.front_mask);
                    self.gl.stencil_mask_separate(BACK, stencil.back_mask);
                    self.gl.stencil_func_separate(
                        FRONT,
                        stencil.front.func as _,
                        stencil.front.sref,
                        stencil.front.mask,
                    );
                    self.gl.stencil_func_separate(
                        BACK,
                        stencil.back.func as _,
                        stencil.back.sref,
                        stencil.back.mask,
                    );
                    self.gl.stencil_op_separate(
                        FRONT,
                        stencil.front_stencil_op[0] as _,
                        stencil.front_stencil_op[1] as _,
                        stencil.front_stencil_op[2] as _,
                    );
                    self.gl.stencil_op_separate(
                        BACK,
                        stencil.back_stencil_op[0] as _,
                        stencil.back_stencil_op[1] as _,
                        stencil.back_stencil_op[2] as _,
                    )
                }
                None => self.gl.disable(STENCIL_TEST),
            }
        }

        draw_cb(Drawable {
            ctx: self,
            pipeline,
            current_program: pipeline.program.clone(),
        });

        // disable vertex attribs
        for i in 0..pipeline.vertex_attributes.len() {
            unsafe { self.gl.disable_vertex_attrib_array(i as _) }
        }
    }

    /// Bind a buffer object to the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn bind_array_buffer(&mut self, buffer: impl BindableBuffer) {
        unsafe { buffer.bind(ARRAY_BUFFER, &self.gl) }
    }

    /// Unbind the currently bound buffer object from the array buffer binding point, `GL_ARRAY_BUFFER`.
    pub fn unbind_array_buffer(&mut self) {
        unsafe { self.gl.bind_buffer(ARRAY_BUFFER, None) };
    }

    /// Bind a buffer object to the index buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn bind_index_buffer(&mut self, buffer: impl BindableBuffer) {
        unsafe { buffer.bind(ELEMENT_ARRAY_BUFFER, &self.gl) }
    }

    /// Unbind the currently bound buffer object from the array buffer binding point, `GL_ELEMENT_ARRAY_BUFFER`.
    pub fn unbind_index_buffer(&mut self) {
        unsafe { self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, None) };
    }

    /// Bind a buffer object. The binding point will be determined using `BufferHandle::buffer_type`.
    pub fn bind_any_buffer(&mut self, buffer: &BufferHandle) {
        let target = match buffer.ty() {
            BufferType::ArrayBuffer => ARRAY_BUFFER,
            BufferType::ElementArrayBuffer => ELEMENT_ARRAY_BUFFER,
        };
        unsafe { buffer.bind(target, &self.gl) }
    }

    /// Clear specified buffers.
    pub fn clear(&self, mask: ClearFlags) {
        unsafe {
            self.gl.clear(mask.bits());
        }
    }

    /// Set the clear color.
    pub fn set_clear_color(&self, color: [f32; 4]) {
        unsafe { self.gl.clear_color(color[0], color[1], color[2], color[3]) };
    }

    /// Set the depth clear value.
    pub fn set_depth_clear(&self, value: f32) {
        unsafe { self.gl.clear_depth_f32(value) };
    }

    /// Set the stencil clear value.
    pub fn set_stencil_clear(&self, value: i32) {
        unsafe { self.gl.clear_stencil(value) };
    }

    /// Set the viewport position & dimensions.
    pub fn set_viewport(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { self.gl.viewport(x, y, w, h) };
    }
}

/// A pipeline draw context.
pub struct Drawable<'a> {
    ctx: &'a mut ManagedContext,
    pipeline: &'a RenderPipeline,
    current_program: Arc<ShaderProgram>,
}

impl<'a> Drawable<'a> {
    /// Set scissor rect
    pub fn set_scissor(&self, x: i32, y: i32, w: i32, h: i32) {
        unsafe { self.ctx.gl.scissor(x, y, w, h) }
    }

    /// Set a float4 uniform on the currently applied pipeline.
    pub fn set_uniform_float4(&self, name: &str, value: &[f32; 4]) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_4_f32(loc.as_ref(), value[0], value[1], value[2], value[3]);
        }
    }

    /// Set a float3 uniform on the currently applied pipeline.
    pub fn set_uniform_float3(&self, name: &str, value: &[f32; 3]) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_3_f32(loc.as_ref(), value[0], value[1], value[2]);
        }
    }

    /// Set a float3 uniform on the currently applied pipeline.
    pub fn set_uniform_float2(&self, name: &str, value: &[f32; 2]) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx.gl.uniform_2_f32(loc.as_ref(), value[0], value[1]);
        }
    }

    /// Set a float1 uniform on the currently applied pipeline.
    pub fn set_uniform_float1(&self, name: &str, value: f32) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx.gl.uniform_1_f32(loc.as_ref(), value);
        }
    }

    /// Set a int4 uniform on the currently applied pipeline.
    pub fn set_uniform_int4(&self, name: &str, value: &[i32; 4]) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_4_i32(loc.as_ref(), value[0], value[1], value[2], value[3]);
        }
    }

    /// Set a int3 uniform on the currently applied pipeline.
    pub fn set_uniform_int3(&self, name: &str, value: &[i32; 3]) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_3_i32(loc.as_ref(), value[0], value[1], value[2]);
        }
    }

    /// Set a int3 uniform on the currently applied pipeline.
    pub fn set_uniform_int2(&self, name: &str, value: &[i32; 2]) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx.gl.uniform_2_i32(loc.as_ref(), value[0], value[1]);
        }
    }

    /// Set a int1 uniform on the currently applied pipeline.
    pub fn set_uniform_int1(&self, name: &str, value: i32) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx.gl.uniform_1_i32(loc.as_ref(), value);
        }
    }

    /// Set a mat2 uniform on the currently applied pipeline.
    /// If you're not sure what `transpose` means, simply make it false.
    pub fn set_uniform_mat2(&self, name: &str, value: &[f32; 4], transpose: bool) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_matrix_2_f32_slice(loc.as_ref(), transpose, value);
        }
    }

    /// Set a mat3 uniform on the currently applied pipeline.
    /// If you're not sure what `transpose` means, simply make it false.
    pub fn set_uniform_mat3(&self, name: &str, value: &[f32; 9], transpose: bool) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_matrix_3_f32_slice(loc.as_ref(), transpose, value);
        }
    }

    /// Set a mat4 uniform on the currently applied pipeline.
    /// If you're not sure what `transpose` means, simply make it false.
    pub fn set_uniform_mat4(&self, name: &str, value: &[f32; 16], transpose: bool) {
        unsafe {
            let program = self.current_program.program;
            let loc = self.ctx.gl.get_uniform_location(program, name);
            assert!(loc.is_some(), "No such uniform name!");
            self.ctx
                .gl
                .uniform_matrix_4_f32_slice(loc.as_ref(), transpose, value);
        }
    }

    /// Bind vertex buffer(s) and index buffer.
    pub fn apply_bindings(
        &self,
        vertex_buffers: &[impl BindableBuffer],
        index_buffer: Option<impl BindableBuffer>,
    ) {
        // setup vaos
        for (idx, attr) in self.pipeline.vertex_attributes.iter().enumerate() {
            let buffer = &vertex_buffers[attr.buffer_index];
            unsafe {
                buffer.bind(ARRAY_BUFFER, &self.ctx.gl);

                self.ctx.gl.vertex_attrib_pointer_f32(
                    idx as _,
                    attr.size,
                    attr.data_type as _,
                    attr.normalized,
                    attr.stride,
                    attr.offset,
                );
				//self.ctx.gl.vertex_attrib_divisor(idx as _, attr.divisor);
                self.ctx.gl.enable_vertex_attrib_array(idx as _);
            }
        }

        unsafe {
            if let Some(index_buffer) = index_buffer {
                index_buffer.bind(ELEMENT_ARRAY_BUFFER, &self.ctx.gl);
            }
        }
    }

    /// Render primitives using bound vertex data & index data.
    pub fn draw_elements(&mut self, mode: DrawMode, count: u32, ty: DataType, offset: i32) {
        unsafe {
            self.ctx
                .gl
                .draw_elements(mode.to_gl(), count as i32, ty as u32, offset);
        }
    }

    /// Render primitives using bound vertex data & index data, with instancing.
    pub fn draw_elements_instanced(
        &self,
        mode: DrawMode,
        count: u32,
        ty: DataType,
        offset: i32,
        instances: u32,
    ) {
        unsafe {
            self.ctx.gl.draw_elements_instanced(
                mode.to_gl(),
                count as i32,
                ty as u32,
                offset,
                instances as _,
            );
        }
    }

    /// Render primitives using bound vertex data.
    pub fn draw_arrays(&self, mode: DrawMode, first: i32, count: i32) {
        unsafe {
            self.ctx.gl.draw_arrays(mode.to_gl(), first as i32, count);
        }
    }

    /// Render primitives using bound vertex data, with instancing.
    pub fn draw_arrays_instanced(&self, mode: DrawMode, first: i32, count: i32, instances: u32) {
        unsafe {
            self.ctx
                .gl
                .draw_arrays_instanced(mode.to_gl(), first as i32, count, instances as _);
        }
    }
}
