use super::*;

use std::sync::Arc;

/// The two buffer types supported by rapax.
/// `ArrayBuffer` corresponds to `GL_ARRAY_BUFFER` and `ElementArrayBuffer` corresponds to `GL_ELEMENT_ARRAY_BUFFER`.
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum BufferType {
    ArrayBuffer,
    ElementArrayBuffer,
}

/// The buffer usage flag passed when allocating buffer data using `glBufferData`.
/// Variants correspond to `GL_STATIC_DRAW`, `GL_DYNAMIC_DRAW`, and `GL_STREAM_DRAW`, respectively.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum BufferUsage {
    Immutable = 0x88E4,
    Dynamic = 0x88E8,
    Stream = 0x88E0,
}

impl BufferUsage {
    pub fn to_gl(&self) -> u32 {
        *self as u32
    }
}

/// The size of an index buffer's indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum DataType {
    SignedByte = 0x1400,
    UnsignedByte = 0x1401,
    SignedShort = 0x1402,
    UnsignedShort = 0x1403,
    SignedInt = 0x1404,
    UnsignedInt = 0x1405,
    HalfFloat = 0x140b,
    Float = 0x1406,
    Double = 0x140a,
    Fixed = 0x140c,
}

impl DataType {
    pub fn to_gl(&self) -> u32 {
        *self as u32
    }
}

/// A handle to an OpenGL buffer. The internal OpenGL buffer object will be automatically freed on drop.
#[derive(Debug, Clone)]
pub struct BufferHandle {
    gl: Arc<Context>,
    capacity: usize,
    buffer: NativeBuffer,
    ty: BufferType,
}

impl BufferHandle {
    /// Create an array buffer, filling it with the given data slice.
    pub fn array_buffer(ctx: &mut ManagedContext, usage: BufferUsage, data: &[u8]) -> Self {
        let buffer = unsafe {
            let buffer = ctx.gl.create_buffer().unwrap();
            ctx.gl.bind_buffer(ARRAY_BUFFER, Some(buffer));
            ctx.gl
                .buffer_data_u8_slice(ARRAY_BUFFER, data, usage.to_gl());

            buffer
        };

        Self {
            buffer,
            gl: ctx.gl.clone(),
            ty: BufferType::ArrayBuffer,
            capacity: data.len(),
        }
    }

    /// Create an index buffer, filling it with the given data slice.
    pub fn index_buffer(ctx: &mut ManagedContext, usage: BufferUsage, data: &[u8]) -> Self {
        let buffer = unsafe {
            let buffer = ctx.gl.create_buffer().unwrap();
            ctx.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(buffer));
            ctx.gl
                .buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, data, usage.to_gl());

            buffer
        };

        Self {
            buffer,
            gl: ctx.gl.clone(),
            ty: BufferType::ElementArrayBuffer,
            capacity: data.len(),
        }
    }

    /// The capacity of the buffer, in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Reallocate the buffer's underlying storage.
    pub fn realloc(&mut self, usage: BufferUsage, data: &[u8]) {
        match self.buffer_type() {
            BufferType::ArrayBuffer => unsafe {
                self.gl.bind_buffer(ARRAY_BUFFER, Some(self.buffer));

                self.gl
                    .buffer_data_u8_slice(ARRAY_BUFFER, data, usage.to_gl());
            },
            BufferType::ElementArrayBuffer => unsafe {
                self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(self.buffer));

                self.gl
                    .buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, data, usage.to_gl());
            },
        }

        self.capacity = data.len();
    }

    /// Update data in the buffer's data storage.
    /// When updating the entire buffer, consider this function over `realloc`.
    /// This avoids the cost of reallocating the buffer object's data store.
    /// 
    /// ## Panics
    /// The offset and the data being updated must lie inside the buffer.
    pub fn update(&self, offset: i32, data: &[u8]) {
        assert!(
            offset as usize + data.len() <= self.capacity,
            "out of bounds write!"
        );

        match self.buffer_type() {
            BufferType::ArrayBuffer => unsafe {
                self.gl.bind_buffer(ARRAY_BUFFER, Some(self.buffer));
                self.gl.buffer_sub_data_u8_slice(ARRAY_BUFFER, offset, data);
            },
            BufferType::ElementArrayBuffer => unsafe {
                self.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(self.buffer));
                self.gl
                    .buffer_sub_data_u8_slice(ELEMENT_ARRAY_BUFFER, offset, data);
            },
        }
    }

    /// The buffer type, either `ArrayBuffer` or `ElementArrayBuffer`.
    pub fn buffer_type(&self) -> BufferType {
        self.ty
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        unsafe { self.gl.delete_buffer(self.buffer) }
    }
}

impl BufferSource for &BufferHandle {
    fn native_buffer(&self) -> NativeBuffer {
        self.buffer
    }
}

impl BufferSource for NativeBuffer {
    fn native_buffer(&self) -> NativeBuffer {
        *self
    }
}

pub trait BufferSource {
    fn native_buffer(&self) -> NativeBuffer;
}
