use crate::blend::*;
use crate::*;

/// Rendering state descriptor.
#[derive(Debug)]
pub struct RenderPipeline {
    // blend state
    pub(crate) blend_enabled: bool,
    pub(crate) blend_func: (u32, u32),

    // depth test
    pub(crate) depth_enabled: bool,

    // depth write & color write
    pub(crate) depth_write: bool,
    pub(crate) color_write: [bool; 4],

    // pipline program
    pub(crate) program: ShaderProgram,
}

impl RenderPipeline {
    pub fn new(program: ShaderProgram) -> Self {
        Self {
            blend_enabled: false,
            blend_func: (0, 0),

            depth_enabled: false,
            depth_write: false,
            color_write: [true, true, true, true],

            program,
        }
    }

    pub fn with_blend(self, enabled: bool) -> Self {
        Self {
            blend_enabled: enabled,
            ..self
        }
    }

    pub fn with_blend_func(self, src: BlendFactor, dst: BlendFactor) -> Self {
        Self {
            blend_func: (src as u32, dst as u32),
            ..self
        }
    }

    pub fn with_depth(self, enabled: bool) -> Self {
        Self {
            depth_enabled: enabled,
            ..self
        }
    }

    pub fn with_depth_write(self, enabled: bool) -> Self {
        Self {
            depth_write: enabled,
            ..self
        }
    }

    pub fn with_color_write(self, r: bool, g: bool, b: bool, a: bool) -> Self {
        Self {
            color_write: [r, g, b, a],
            ..self
        }
    }

    pub fn program(&self) -> &ShaderProgram {
        &self.program
    }
}
