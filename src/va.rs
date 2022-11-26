use super::*;
use std::sync::Arc;

pub struct VertexArrayObject {
    vao: NativeVertexArray,
    gl: Arc<Context>,
}

impl VertexArrayObject {
    pub fn new(ctx: &mut ManagedContext) -> Self {
        let vao = unsafe { ctx.gl.create_vertex_array().unwrap() };
        Self {
            vao,
            gl: ctx.gl.clone(),
        }
    }

    pub fn attrib_pointer(
        &self,
        ctx: &mut ManagedContext,
        index: u32,
        size: i32,
        data_type: DataType,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        ctx.bind_vertex_array_with(self.vao, |ctx| {
            unsafe {
                self.gl.vertex_attrib_pointer_f32(
                    index,
                    size,
                    data_type as _,
                    normalized,
                    stride,
                    offset,
                );
            }
        });
    }

    pub fn enable_attrib(&self, ctx: &mut ManagedContext, index: u32) {
        ctx.bind_vertex_array_with(self.vao, |ctx| {
            unsafe {
                self.gl.enable_vertex_attrib_array(index);
            }
        });
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