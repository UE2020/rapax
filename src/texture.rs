use super::*;
use std::sync::Arc;

/// Specifies an OpenGL texture format.
///
/// The availability of texture formats depends on the platform being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureFormat {
    /// Three-component format with 8 bits per component (24 bits total).
    RGB = RGB,
    /// Four-component format with 8 bits per component (32 bits total).
    RGBA = RGBA,
    /// Three-component format with 5 bits for red, 6 bits for green, and 5 bits for blue (16 bits total).
    RGB565 = RGB565,
    /// Four-component format with 4 bits per component (16 bits total).
    RGBA4444 = RGBA4,
    /// Four-component format with 5 bits for red, 5 bits for green, 5 bits for blue, and 1 bit for alpha (16 bits total).
    RGBA5551 = RGB5_A1,
    /// Four-component format with 8 bits per component (32 bits total).
    RGBA8888 = RGBA8,
    /// Four-component format with 10 bits for red, 10 bits for green, 10 bits for blue, and 2 bits for alpha (32 bits total).
    #[allow(non_camel_case_types)]
    RGB10_A2 = RGB10_A2,
    /// Single-component format with 8 bits (usually used for grayscale textures).
    R8 = R8,
    /// Single-component format with 16 bits.
    R16 = R16,
    /// Two-component format with 8 bits per component (16 bits total).
    RG8 = RG8,
    /// Two-component format with 16 bits per component (32 bits total).
    RG16 = RG16,
    /// Four-component format with 32-bit floating-point precision per component (128 bits total).
    RGBA32F = RGBA32F,
    /// Four-component format with 16-bit floating-point precision per component (64 bits total).
    RGBA16F = RGBA16F,
    /// Single-component format used for depth textures.
    DepthComponent = DEPTH_COMPONENT,
    /// Two-component format combining depth and stencil information.
    DepthStencil = DEPTH24_STENCIL8,
}

/// Specifies the wrapping behavior of an axis of a texture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureWrap {
    MirroredRepeat = MIRRORED_REPEAT,
    ClampToBorder = CLAMP_TO_BORDER,
}

/// Specifies a potential texture filtering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureFilteringMode {
    /// Picks the nearest pixel.
    Nearest = NEAREST,
    /// Averages pixels in the surrounding area.
    Linear = LINEAR,
    /// Takes the nearest mipmap to match the pixel size and uses nearest neighbor interpolation for texture sampling.
    NearestMipmapNearest = NEAREST_MIPMAP_NEAREST,
}

/// A handle to an OpenGL texture object. The internal OpenGL program object will be automatically freed on drop.
#[derive(Debug)]
pub struct TextureHandle {
    pub(crate) texture: NativeTexture,
    gl: Arc<Context>,
}

impl TextureHandle {
    /// Create a new texture.
    pub fn new(
        ctx: &mut ManagedContext,
        wrapping_mode_s: TextureWrap,
        wrapping_mode_t: TextureWrap,
        min_filter: TextureFilteringMode,
        mag_filter: TextureFilteringMode,
    ) -> Result<Self, String> {
        let texture = unsafe {
            let texture = ctx.gl.create_texture()?;
            ctx.gl.bind_texture(TEXTURE_2D, Some(texture));
            ctx.gl
                .tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_S, wrapping_mode_s as _);
            ctx.gl
                .tex_parameter_i32(TEXTURE_2D, TEXTURE_WRAP_T, wrapping_mode_t as _);
            ctx.gl
                .tex_parameter_i32(TEXTURE_2D, TEXTURE_MAG_FILTER, mag_filter as _);
            ctx.gl
                .tex_parameter_i32(TEXTURE_2D, TEXTURE_MIN_FILTER, min_filter as _);
            ctx.gl.bind_texture(TEXTURE_2D, None);
            texture
        };
        Ok(Self {
            texture: texture,
            gl: ctx.gl.clone(),
        })
    }

    /// Set the TEXTURE_BORDER_COLOR texture parameter.
    pub fn set_border_color(&self, ctx: &mut ManagedContext, color: [f32; 4]) {
        unsafe {
            ctx.gl.bind_texture(TEXTURE_2D, Some(self.texture));
            ctx.gl
                .tex_parameter_f32_slice(TEXTURE_2D, TEXTURE_BORDER_COLOR, &color);
            ctx.gl.bind_texture(TEXTURE_2D, None);
        }
    }

    /// Upload/allocate 2D texture data and receive a [`Texture2D`] instance.
    pub fn allocate_2d_data(
        self,
        ctx: &mut ManagedContext,
        data: Option<&[u8]>,
        format: TextureFormat,
        width: i32,
        height: i32,
        ty: DataType,
    ) -> Texture2D {
        unsafe {
            ctx.gl.bind_texture(TEXTURE_2D, Some(self.texture));
            ctx.gl.tex_image_2d(
                TEXTURE_2D,
                0,
                format as _,
                width,
                height,
                0,
                format as _,
                ty as _,
                data,
            );
            ctx.gl.bind_texture(TEXTURE_2D, None);
            Texture2D(self)
        }
    }
}

/// A 2D texture in GPU memory.
#[derive(Debug)]
pub struct Texture2D(TextureHandle);

/// A wrapper around a native OpenGL texture.
#[derive(Debug)]
pub struct BindableNativeTexture {
    texture: NativeTexture,
    target: u32,
}

impl BindableNativeTexture {
    pub fn new(texture: NativeTexture, target: u32) -> Self {
        Self { texture, target }
    }
}

impl BindableTexture for &BindableNativeTexture {
    unsafe fn bind(&self, target: u32, gl: &Context) {
        gl.bind_texture(target, Some(self.texture));
    }

    fn texture_target_hint(&self) -> u32 {
        self.target
    }
}

impl Texture2D {
    /// Generate texture mipmaps, should be called when texture data changes.
    pub fn generate_mipmaps(&self, ctx: &mut ManagedContext) {
        unsafe {
            ctx.gl.bind_texture(TEXTURE_2D, Some(self.0.texture));
            ctx.gl.generate_mipmap(TEXTURE_2D);
            ctx.gl.bind_texture(TEXTURE_2D, None);
        }
    }
}

impl BindableTexture for Texture2D {
    unsafe fn bind(&self, target: u32, gl: &Context) {
        gl.bind_texture(target, Some(self.0.texture));
    }

    fn texture_target_hint(&self) -> u32 {
        TEXTURE_2D
    }
}

pub trait BindableTexture {
    unsafe fn bind(&self, target: u32, gl: &Context);

    fn texture_target_hint(&self) -> u32;
}

impl Drop for TextureHandle {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_texture(self.texture);
        }
    }
}
