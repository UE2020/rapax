use super::*;
use std::sync::Arc;

/// A handle to an OpenGL shader program. The internal OpenGL program object will be automatically freed on drop.
#[derive(Debug)]
pub struct ShaderProgram {
    program: NativeProgram,
    gl: Arc<Context>,
}

impl ShaderProgram {
    /// Create a new program, using sources passed in as strings.
    pub fn new(
        ctx: &ManagedContext,
        vertex_shader_source: &str,
        fragment_shader_source: &str,
    ) -> Self {
        let shader = compile_shader(&ctx.gl, vertex_shader_source, fragment_shader_source);
        Self {
            program: shader,
            gl: ctx.gl.clone(),
        }
    }
}

impl ProgramSource for ShaderProgram {
    fn native_program(&self) -> NativeProgram {
        self.program
    }
}

impl ProgramSource for NativeProgram {
    fn native_program(&self) -> NativeProgram {
        *self
    }
}

pub trait ProgramSource {
    fn native_program(&self) -> NativeProgram;
}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        unsafe {
            self.gl.delete_program(self.program);
        }
    }
}

fn compile_shader(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> NativeProgram {
    unsafe {
        let program = gl.create_program().expect("Cannot create program"); // compile and link shader program

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.shader_source(shader, &format!("{}\n{}", "#version 330", shader_source));
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                // TODO: use Result instead of panicking
                std::panic::panic_any(gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            std::panic::panic_any(gl.get_program_info_log(program));
        }

        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }

        program
    }
}
