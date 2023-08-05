use super::*;

use std::sync::Arc;

/// The two buffer types supported by rapax.
/// `ArrayBuffer` corresponds to `GL_ARRAY_BUFFER` and `ElementArrayBuffer` corresponds to `GL_ELEMENT_ARRAY_BUFFER`.
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
pub enum BufferType {
    ArrayBuffer = ARRAY_BUFFER,
    ElementArrayBuffer = ELEMENT_ARRAY_BUFFER,
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
    SignedByte = BYTE,
    UnsignedByte = UNSIGNED_BYTE,
    SignedShort = SHORT,
    UnsignedShort = UNSIGNED_SHORT,
    SignedInt = INT,
    UnsignedInt = UNSIGNED_INT,
    HalfFloat = HALF_FLOAT,
    Float = FLOAT,
    Double = DOUBLE,
    Fixed = FIXED,
}

impl DataType {
    pub fn to_gl(&self) -> u32 {
        *self as u32
    }

    pub fn sizeof(&self) -> usize {
        match *self {
            Self::SignedByte => 1,
            Self::UnsignedByte => 1,
            Self::SignedShort => 2,
            Self::UnsignedShort => 2,
            Self::SignedInt => 4,
            Self::UnsignedInt => 4,
            Self::HalfFloat => 2,
            Self::Float => 4,
            Self::Double => 8,
            Self::Fixed => 4,
        }
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
    pub fn array_buffer(
        ctx: &ManagedContext,
        usage: BufferUsage,
        data: &[u8],
    ) -> Result<Self, String> {
        let buffer = unsafe {
            let buffer = ctx.gl.create_buffer()?;
            ctx.gl.bind_buffer(ARRAY_BUFFER, Some(buffer));
            ctx.gl
                .buffer_data_u8_slice(ARRAY_BUFFER, data, usage.to_gl());

            buffer
        };

        Ok(Self {
            buffer,
            gl: ctx.gl.clone(),
            ty: BufferType::ArrayBuffer,
            capacity: data.len(),
        })
    }

    /// Create an index buffer, filling it with the given data slice.
    pub fn index_buffer(
        ctx: &mut ManagedContext,
        usage: BufferUsage,
        data: &[u8],
    ) -> Result<Self, String> {
        let buffer = unsafe {
            let buffer = ctx.gl.create_buffer()?;
            ctx.gl.bind_buffer(ELEMENT_ARRAY_BUFFER, Some(buffer));
            ctx.gl
                .buffer_data_u8_slice(ELEMENT_ARRAY_BUFFER, data, usage.to_gl());

            buffer
        };

        Ok(Self {
            buffer,
            gl: ctx.gl.clone(),
            ty: BufferType::ElementArrayBuffer,
            capacity: data.len(),
        })
    }

    /// The capacity of the buffer, in bytes.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Reallocate the buffer's underlying storage.
    pub fn realloc(&mut self, usage: BufferUsage, data: &[u8]) {
        match self.ty() {
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

        match self.ty() {
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
    pub fn ty(&self) -> BufferType {
        self.ty
    }
}

impl Drop for BufferHandle {
    fn drop(&mut self) {
        unsafe { self.gl.delete_buffer(self.buffer) }
    }
}

impl BindableBuffer for &BufferHandle {
    unsafe fn bind(&self, target: u32, gl: &Context) {
        assert_eq!(
            target,
            self.ty() as _,
            "Attempted to bind buffer to invalid binding point"
        );
        gl.bind_buffer(target, Some(self.buffer));
    }
}

impl BindableBuffer for NativeBuffer {
    unsafe fn bind(&self, target: u32, gl: &Context) {
        gl.bind_buffer(target, Some(*self));
    }
}

pub trait BindableBuffer {
    unsafe fn bind(&self, target: u32, gl: &Context);
}
