use super::*;
use std::sync::Arc;

mod tex_2d;
pub use tex_2d::*;

/// Specifies an internal OpenGL texture format.
///
/// The availability of texture formats depends on the platform being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum InternalTextureFormat {
    /// Alpha format.
    Alpha = ALPHA,
    /// Luminance format.
    Luminance = LUMINANCE,
    /// Luminance alpha format.
    LuminanceAlpha = LUMINANCE_ALPHA,
    /// 3-bit red, 3-bit green, 2-bit blue format.
    R3G3B2 = R3_G3_B2,
    /// RGB format.
    Rgb = RGB,
    /// 4-bit RGB format.
    Rgb4 = RGB4,
    /// 5-bit RGB format.
    Rgb5 = RGB5,
    /// 8-bit RGB format.
    Rgb8 = RGB8,
    /// 10-bit RGB format.
    Rgb10 = RGB10,
    /// 12-bit RGB format.
    Rgb12 = RGB12,
    /// 16-bit RGB format.
    Rgb16 = RGB16,
    /// RGBA format.
    Rgba = RGBA,
    /// 2-bit RGBA format.
    Rgba2 = RGBA2,
    /// 4-bit RGBA format.
    Rgba4 = RGBA4,
    /// 5-bit RGB format with 1-bit alpha.
    Rgb5A1 = RGB5_A1,
    /// 8-bit RGBA format.
    Rgba8 = RGBA8,
    /// 10-bit RGB format with 2-bit alpha.
    Rgb10A2 = RGB10_A2,
    /// 12-bit RGBA format.
    Rgba12 = RGBA12,
    /// 16-bit RGBA format.
    Rgba16 = RGBA16,
}


/// Specifies a supported OpenGL texture format.
///
/// The availability of texture formats depends on the platform being used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum TextureFormat {
    Red = RED,
    Green = GREEN,
    Blue = BLUE,
    Alpha = ALPHA,
    Rgb = RGB,
    Rgba = RGBA,
    Luminance = LUMINANCE,
    LuminanceAlpha = LUMINANCE_ALPHA,
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
        internal_format: InternalTextureFormat,
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
                internal_format as _,
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
