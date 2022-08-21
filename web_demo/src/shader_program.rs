use web_sys::WebGl2RenderingContext as GL;
use web_sys as web;

pub struct ShaderProgram {
    pub vert: web::WebGlShader,
    pub frag: web::WebGlShader,
    pub handle: web::WebGlProgram
}

impl ShaderProgram {

    pub fn load(ctx: &web::WebGl2RenderingContext, vert: &str, frag: &str) -> Result<Self, String> {
        let vert = compile_shader(&ctx, vert, GL::VERTEX_SHADER)?;
        let frag = compile_shader(&ctx, frag, GL::FRAGMENT_SHADER)?;
        let handle = compile_program(&ctx, &vert, &frag)?;
        Ok(ShaderProgram { vert, frag, handle })
    }

}

pub fn compile_shader(ctx: &web::WebGl2RenderingContext, source: &str, shader_type: u32) -> Result<web::WebGlShader, String> {
    let shader = ctx.create_shader(shader_type)
        .ok_or_else(|| "Failed to create shader".to_string() )?;

    ctx.shader_source(&shader, source);
    ctx.compile_shader(&shader);

    let success = ctx.get_shader_parameter(&shader, GL::COMPILE_STATUS);
    if !success.as_bool().unwrap_or(false) {
        let log = ctx.get_shader_info_log(&shader)
            .ok_or_else(|| "Failed to get shader info log".to_string() )?;

        Err(format!("Failed to compile shader: {}", log))
    } else {
        Ok(shader)
    }
}

pub fn compile_program(ctx: &web::WebGl2RenderingContext, vert: &web::WebGlShader, frag: &web::WebGlShader) -> Result<web::WebGlProgram, String> {
    let program = ctx.create_program()
        .ok_or_else(|| "Failed to create program".to_string() )?;

    ctx.attach_shader(&program, vert);
    ctx.attach_shader(&program, frag);

    ctx.link_program(&program);

    let success = ctx.get_program_parameter(&program, GL::LINK_STATUS);
    if !success.as_bool().unwrap_or(false) {
        let log = ctx.get_program_info_log(&program)
            .ok_or_else(|| "Failed to get program info log".to_string() )?;

        Err(format!("Failed to link shader program: {}", log))
    } else {
        Ok(program)
    }
}
