use crate::*;

use std::sync::Arc;

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

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[repr(u32)]
pub enum StencilOp {
	Keep = KEEP,
	Zero = ZERO,
	Replace = REPLACE,
	Increment = INCR,
	IncrementWrap = INCR_WRAP,
	Decrement = DECR,
	DecrementWrap = DECR_WRAP,
	Invert = INVERT,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VertexAttributeDescriptor {
	pub(crate) buffer_index: usize,
    pub(crate) size: i32,
    pub(crate) data_type: DataType,
    pub(crate) normalized: bool,
    pub(crate) stride: i32,
    pub(crate) offset: i32,

    pub(crate) divisor: u32,
}

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
    pub fn with_vertex_attribute(
        self,
		buffer_index: usize,
        size: i32,
        data_type: DataType,
        normalized: bool,
        stride: i32,
        offset: i32,
        divisor: u32,
    ) -> Self {
        let mut vertex_attributes = self.vertex_attributes;
        vertex_attributes.push(VertexAttributeDescriptor {
			buffer_index,
            size,
            data_type,
            normalized,
            stride,
            offset,
            divisor,
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

	pub fn with_stencil(self, stencil: Option<StencilState>) -> Self {
		Self {
			stencil_state: stencil,
			..self
		}
	}
}
