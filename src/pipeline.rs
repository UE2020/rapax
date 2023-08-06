use crate::*;

use std::sync::Arc;

/// A stencil function.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u32)]
pub enum StencilFunc {
    Never = NEVER,
    Less = LESS,
    LessThanOrEqual = LEQUAL,
    Greater = GREATER,
    GreaterThanOrEqual = GEQUAL,
    Equal = EQUAL,
    NotEqual = NOTEQUAL,
    Always = ALWAYS,
}

/// A stencil operation.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u32)]
pub enum StencilOp {
    /// The currently stored stencil value is kept.
    Keep = KEEP,
    /// The stencil value is set to 0.
    Zero = ZERO,
    /// The stencil value is replaced with the reference value.
    Replace = REPLACE,
    /// The stencil value is increased by 1 if it is lower than the maximum value.
    Increment = INCR,
    /// Same as [`StencilOp::Increment`], but wraps it back to 0 as soon as the maximum value is exceeded.
    IncrementWrap = INCR_WRAP,
    /// The stencil value is decreased by 1 if it is higher than the minimum value.
    Decrement = DECR,
    /// Same as [`StencilOp::Decrement`], but wraps it to the maximum value if it ends up lower than 0.
    DecrementWrap = DECR_WRAP,
    /// Bitwise inverts the current stencil buffer value.
    Invert = INVERT,
}

/// Vertex attribute descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VertexAttributeDescriptor {
    pub buffer_index: usize,
    pub size: i32,
    pub ty: DataType,
    pub normalized: bool,
    pub stride: i32,
    pub offset: i32,
    pub divisor: u32,
}

/// Stencil function state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StencilFuncState {
    pub mask: u32,
    pub func: StencilFunc,
    pub sref: i32,
}

impl Default for StencilFuncState {
    fn default() -> Self {
        Self {
            mask: 0xFF,
            func: StencilFunc::Always,
            sref: 1,
        }
    }
}

/// Stencil state descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StencilState {
    pub front_mask: u32,
    pub back_mask: u32,

    pub front: StencilFuncState,
    pub back: StencilFuncState,
    pub front_stencil_op: [StencilOp; 3],
    pub back_stencil_op: [StencilOp; 3],
}

impl Default for StencilState {
    fn default() -> Self {
        Self {
            front_mask: 0xFF,
            back_mask: 0xFF,

            front: Default::default(),
            back: Default::default(),

            front_stencil_op: [StencilOp::Keep; 3],
            back_stencil_op: [StencilOp::Keep; 3],
        }
    }
}

/// Rendering state descriptor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderPipeline {
    // blend state
    pub(crate) blend_enabled: bool,
    pub(crate) blend_func: (u32, u32),

    // scissor state
    pub(crate) scissor_enabled: bool,

    // stencil state
    pub(crate) stencil_state: Option<StencilState>,

    // depth test
    pub(crate) depth_enabled: bool,

    // depth write & color write
    pub(crate) depth_write: bool,
    pub(crate) color_write: [bool; 4],

    // pipeline program
    pub(crate) program: Arc<ShaderProgram>,

    pub(crate) vertex_attributes: Vec<VertexAttributeDescriptor>,
}

impl RenderPipeline {
    /// Create a new pipeline using the given shader program.
    pub fn new(program: ShaderProgram) -> Self {
        Self {
            blend_enabled: false,
            blend_func: (0, 0),

            scissor_enabled: false,
            stencil_state: None,

            depth_enabled: false,
            depth_write: false,
            color_write: [true, true, true, true],

            program: Arc::new(program),

            vertex_attributes: vec![],
        }
    }

    /// Add a vertex attribute to the pipeline.
    pub fn with_vertex_attribute(self, attr: VertexAttributeDescriptor) -> Self {
        let mut vertex_attributes = self.vertex_attributes;
        vertex_attributes.push(attr);

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

    /// Set the stencil state.
    pub fn with_stencil(self, stencil: Option<StencilState>) -> Self {
        Self {
            stencil_state: stencil,
            ..self
        }
    }
}
