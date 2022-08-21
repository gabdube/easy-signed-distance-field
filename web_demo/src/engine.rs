//! A quick and dirty webgl2 engine to render the sdf
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys as web;
use web_sys::WebGl2RenderingContext as GL;
use easy_signed_distance_field as sdf;

use super::shader_program::ShaderProgram;
use std::mem;

#[repr(C)]
#[derive(Copy, Clone, Default)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
}

pub struct Engine {
    canvas: web::HtmlCanvasElement,
    ctx: web::WebGl2RenderingContext,
    program: ShaderProgram,
    texture: Option<web::WebGlTexture>,
    _vbo: web::WebGlBuffer,
    vao: web::WebGlVertexArrayObject,
}

impl Engine {

    pub fn init(ctx: &JsValue) -> Option<Self> {
        let ctx = ctx.clone().dyn_into::<web::WebGl2RenderingContext>().ok()?;
        let canvas = ctx.canvas()?.dyn_into::<web::HtmlCanvasElement>().ok()?;

        // General state setup
        ctx.clear_color(1.0, 1.0, 1.0, 1.0);
        ctx.enable(GL::BLEND);
        ctx.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        ctx.pixel_storei(GL::UNPACK_ALIGNMENT, 1);

        // Shaders setup
        static GUI_TEXT_VERT: &'static str = include_str!("gui_text.vert.glsl");
        static GUI_TEXT_FRAG: &'static str = include_str!("gui_text.frag.glsl");
        let program = match ShaderProgram::load(&ctx, GUI_TEXT_VERT, GUI_TEXT_FRAG) {
            Ok(p) => p,
            Err(e) => { crate::log(&e); return None; }
        };

        // Buffer setup
        let vertex_size = 6 * mem::size_of::<Vertex>();
        let vbo = ctx.create_buffer()?;
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        ctx.buffer_data_with_i32(GL::ARRAY_BUFFER, vertex_size as _, GL::DYNAMIC_DRAW);
        ctx.buffer_sub_data_with_i32_and_u8_array(GL::ARRAY_BUFFER, 0, &Self::vertex_data());
        ctx.bind_buffer(GL::ARRAY_BUFFER, None);

        // Vao setup
        const POSITION_ATTR_LOC: u32 = 0; 
        const UV_ATTR_LOC: u32 = 1;
        let vao = ctx.create_vertex_array()?;

        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));
        ctx.bind_vertex_array(Some(&vao));

        ctx.enable_vertex_attrib_array(POSITION_ATTR_LOC);
        ctx.vertex_attrib_pointer_with_i32(POSITION_ATTR_LOC, 2, GL::FLOAT, false, 16, 0);

        ctx.enable_vertex_attrib_array(UV_ATTR_LOC);
        ctx.vertex_attrib_pointer_with_i32(UV_ATTR_LOC, 2, GL::FLOAT, false, 16, 8);

        ctx.bind_buffer(GL::ARRAY_BUFFER, None);
        ctx.bind_vertex_array(None);


        let engine = Engine {
            canvas,
            ctx,
            program,
            texture: None,
            _vbo: vbo,
            vao,
        };

        engine.update_render_mid_value(0.5);

        Some(engine)
    }

    pub fn render(&mut self, _metrics: &sdf::Metrics, sdf: &sdf::SdfRaster) {
        // Resize viewport
        self.resize();

        // Upload texture
        let ctx = &self.ctx;
        if let Some(t) = self.texture.take() {
            ctx.delete_texture(Some(&t));
        }

        let sdf_bitmap = sdf::sdf_to_bitmap(sdf);
        //crate::log_value(&format!("{:?} {}", (sdf.width, sdf.height), sdf.width * sdf.height), false);

        let texture = ctx.create_texture().unwrap();
        ctx.bind_texture(GL::TEXTURE_2D, Some(&texture));
        ctx.tex_storage_2d(GL::TEXTURE_2D, 1, GL::R8, sdf.width as i32, sdf.height as i32);
        ctx.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            0,    // Level
            0, 0, // Offset
            sdf.width as i32, sdf.height as i32, // Size,
            GL::RED,
            GL::UNSIGNED_BYTE,
            Some(sdf_bitmap.buffer.as_slice())
        ).unwrap();
        ctx.bind_texture(GL::TEXTURE_2D, None);
        self.texture = Some(texture);

        // Draw 
        ctx.clear(GL::DEPTH_BUFFER_BIT | GL::COLOR_BUFFER_BIT);
        ctx.use_program(Some(&self.program.handle));

        ctx.bind_vertex_array(Some(&self.vao));
        ctx.bind_texture(GL::TEXTURE_2D, self.texture.as_ref());
        ctx.draw_arrays(GL::TRIANGLES, 0, 6);

        ctx.bind_texture(GL::TEXTURE_2D, None);
        ctx.bind_vertex_array(None);
        ctx.use_program(None);
    }

    pub fn update_render_mid_value(&self, value: f32) {
        let ctx = &self.ctx;
        ctx.use_program(Some(&self.program.handle));
        let offset_loc = ctx.get_uniform_location(&self.program.handle, "midValue");
        ctx.uniform1f(offset_loc.as_ref(), value);
        ctx.use_program(None);
    }

    fn resize(&mut self) {
        let canvas = &self.canvas;
        let width = canvas.width();
        let height = canvas.height();
        self.ctx.viewport(0, 0, width as i32, height as i32);
    }

    fn vertex_data() -> &'static [u8] {
        // Canvas size will always match the character size so we can use absolute values
        static VERTEX: [Vertex; 6] = [
            Vertex { position: [ -1.0, -1.0 ], uv: [ 0.0, 1.0 ] },
            Vertex { position: [ -1.0, 1.0 ], uv: [ 0.0, 0.0 ] },
            Vertex { position: [ 1.0, 1.0 ], uv: [ 1.0, 0.0 ] },
            Vertex { position: [ -1.0, -1.0 ], uv: [ 0.0, 1.0 ] },
            Vertex { position: [ 1.0, 1.0 ], uv: [ 1.0, 0.0 ] },
            Vertex { position: [ 1.0, -1.0 ], uv: [ 1.0, 1.0 ] },
        ];

        unsafe { VERTEX.align_to::<u8>().1 }
    }

}

unsafe impl Send for Engine {}
unsafe impl Sync for Engine {}
