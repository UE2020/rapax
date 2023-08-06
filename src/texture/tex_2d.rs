use super::*;

/// A 2D texture in GPU memory.
#[derive(Debug)]
pub struct Texture2D(pub(crate) TextureHandle);

impl Texture2D {
    /// Generate texture mipmaps, should be called when texture data changes.
    pub fn generate_mipmaps(&self, ctx: &mut ManagedContext) {
        unsafe {
            ctx.gl.bind_texture(TEXTURE_2D, Some(self.0.texture));
            ctx.gl.generate_mipmap(TEXTURE_2D);
            ctx.gl.bind_texture(TEXTURE_2D, None);
        }
    }

    /// Upload a sub-image.
    ///
    /// Mipmaps should be regenerated after the texture is modified.
    pub fn write_subimage(
        &self,
        ctx: &mut ManagedContext,
        x_offset: i32,
        y_offset: i32,
        width: i32,
        height: i32,
        format: TextureFormat,
        ty: DataType,
        data: &[u8],
    ) {
        unsafe {
            ctx.gl.bind_texture(TEXTURE_2D, Some(self.0.texture));
            ctx.gl.tex_sub_image_2d(
                TEXTURE_2D,
                0,
                x_offset,
                y_offset,
                width,
                height,
                format as _,
                ty as _,
                PixelUnpackData::Slice(data),
            );
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
