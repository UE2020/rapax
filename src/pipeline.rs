use crate::*;

use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VertexAttributeDescriptor {
    pub(crate) size: i32,
    pub(crate) data_type: DataType,
    pub(crate) normalized: bool,
    pub(crate) stride: i32,
    pub(crate) offset: i32,

    pub(crate) divisor: u32,
}

/// Rendering state descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    pub(crate) program: Arc<ShaderProgram>,

    pub(crate) vertex_attributes: Vec<VertexAttributeDescriptor>,
}

impl RenderPipeline {
    /// Create a new pipeline using the given shader program.
    pub fn new(program: ShaderProgram) -> Self {
        Self {
            blend_enabled: false,
            blend_func: (0, 0),

            depth_enabled: false,
            depth_write: false,
            color_write: [true, true, true, true],

            program: Arc::new(program),

            vertex_attributes: vec![],
        }
    }

    /// Add a vertex attribute to the pipeline.
    pub fn with_vertex_attribute(
        self,
        size: i32,
        data_type: DataType,
        normalized: bool,
        stride: i32,
        offset: i32,
        divisor: u32,
    ) -> Self {
        let mut vertex_attributes = self.vertex_attributes;
        vertex_attributes.push(VertexAttributeDescriptor {
            size,
            data_type,
            normalized,
            stride,
            offset,
            divisor
        });

        Self {
            vertex_attributes,
            ..self
        }
    }

    /// Set the blend state.
    pub fn with_blend(self, enabled: bool) -> Self {
        Self {
            blend_enabled: enabled,
            ..self
        }
    }

    /// Set the blend function.
    pub fn with_blend_func(self, src: BlendFactor, dst: BlendFactor) -> Self {
        Self {
            blend_func: (src as u32, dst as u32),
            ..self
        }
    }

    /// Set the depth state.
    pub fn with_depth(self, enabled: bool) -> Self {
        Self {
            depth_enabled: enabled,
            ..self
        }
    }

    /// Set the depth write state.
    pub fn with_depth_write(self, enabled: bool) -> Self {
        Self {
            depth_write: enabled,
            ..self
        }
    }

    /// Set the color write state, per channel.
    pub fn with_color_write(self, r: bool, g: bool, b: bool, a: bool) -> Self {
        Self {
            color_write: [r, g, b, a],
            ..self
        }
    }

    /// Get a reference to the shader program. Useful for setting uniforms.
    pub fn program(&self) -> &ShaderProgram {
        &self.program
    }
}
