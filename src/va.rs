use super::*;
use std::sync::Arc;

/// A handle to a vertex array.
pub struct VertexArrayObject {
    vao: NativeVertexArray,
    gl: Arc<Context>,
}

impl VertexArrayObject {
    /// Initialize a new vertex array.
    pub fn new(ctx: &mut ManagedContext) -> Self {
        let vao = unsafe { ctx.gl.create_vertex_array().unwrap() };
        Self {
            vao,
            gl: ctx.gl.clone(),
        }
    }

    /// Add a vertex attribute.
    pub fn attrib_pointer(
        &self,
        index: u32,
        size: i32,
        data_type: DataType,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.vertex_attrib_pointer_f32(
                index,
                size,
                data_type as _,
                normalized,
                stride,
                offset,
            );
            self.gl.bind_vertex_array(None);
        }
    }

    /// Enable a vertex attribute using its index.
    pub fn enable_attrib(&self, ctx: &mut ManagedContext, index: u32) {
        unsafe {
            self.gl.bind_vertex_array(Some(self.vao));
            self.gl.enable_vertex_attrib_array(index);
            self.gl.bind_vertex_array(None);
        }
    }
}

impl Drop for VertexArrayObject {
    fn drop(&mut self) {
        unsafe { self.gl.delete_vertex_array(self.vao) };
    }
}

pub trait VertexArraySource {
    fn native_vertex_array(&self) -> NativeVertexArray;
}

impl VertexArraySource for &VertexArrayObject {
    fn native_vertex_array(&self) -> NativeVertexArray {
        self.vao
    }
}

impl VertexArraySource for NativeVertexArray {
    fn native_vertex_array(&self) -> NativeVertexArray {
        *self
    }
}
