use wasm_bindgen::prelude::*;
use wasm_bindgen::{JsCast, Clamped};

use web_sys as web;
use easy_signed_distance_field as sdf;

use console_error_panic_hook;

use std::sync::Mutex;

mod shader_program;
mod engine;
use engine::Engine;


static FONT: Mutex<Option<sdf::Font>> = Mutex::new(None);
static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

#[wasm_bindgen]
pub fn startup(render_ctx: &JsValue) {
    set_panic_hook();

    let mut engine = ENGINE.lock().unwrap();
    *engine = Engine::init(render_ctx); 
}

#[wasm_bindgen]
pub fn parse_font(data: &JsValue) {
    let data_array = js_sys::Uint8Array::new(&data);
    let data_bytes = data_array.to_vec();
    match sdf::Font::from_bytes(data_bytes.as_slice(), Default::default()) {
        Ok(font) => {
            if let Some(name) = font.name() {
                log(&format!("Loaded font {:?}", name));
            }
            
            *FONT.lock().unwrap() = Some(font);
        },
        Err(e) => {
            log(&format!("Failed to read font file: {}", e));
        }
    }
}

#[wasm_bindgen]
pub fn update_render_mid_value(value: f32) {
    let engine = ENGINE.lock().unwrap();
    if let Some(engine) = engine.as_ref() {
        engine.update_render_mid_value(value);
    }
}

#[wasm_bindgen]
pub fn compute_fixed_height(character: &str, height: f32) -> Option<f32> {
    let font = FONT.lock().unwrap();
    if let Some(font) = font.as_ref() {
        let c = character.chars().next()?;
        font.char_height_to_font_size(c, height)
    } else {
        None
    }
}

#[wasm_bindgen]
pub fn render(output_ctx: &JsValue, character: &str, size: f32, spread: f32) -> Option<u32> {
    if size < 1.0 {
        return None;
    }
    
    let _font = FONT.lock().unwrap();
    let font = _font.as_ref()?;

    let mut engine = ENGINE.lock().unwrap();
    let engine = engine.as_mut()?;

    let c = character.chars().next()?;
    let (metrics, sdf) = font.sdf_generate(size, 2, spread, c)?;

    let output_ctx = output_ctx.clone().dyn_into::<web::CanvasRenderingContext2d>().ok()?;
    let [width, height] = ctx_size(&output_ctx)?;

    // Display the sdf output
    let image_data = create_image(&sdf.buffer, sdf.width, sdf.height)?;
    output_ctx.put_image_data(&image_data, ((width - sdf.width) / 2) as f64, ((height - sdf.height) / 2) as f64).ok()?;

    // Renders the sdf using webgl
    engine.render(&metrics, &sdf);

    Some(0)
}

/// Creates an image data object from a float array
fn create_image(data: &[f32], width: u32, height: u32) -> Option<web::ImageData> {
    let total_buffer_size = (width * height * 4) as usize;
    let mut buffer: Vec<u8> = vec![0u8; total_buffer_size];
    for x in 0..width {
        for y in 0..height {
            let index = (x + (width * y)) as usize;
            let value = (data[index] * 255.0).clamp(0.0, 255.0) as u8;

            let buffer_index = index*4;
            buffer[buffer_index] = value;
            buffer[buffer_index+1] = value;
            buffer[buffer_index+2] = value;
            buffer[buffer_index+3] = 255;
        }
    }

    let buffer = Clamped(buffer.as_slice());
    match web::ImageData::new_with_u8_clamped_array_and_sh(buffer, width as _, height as _) {
        Ok(data) => Some(data),
        Err(err) => {
            log_js(&err);
            None
        }
    }
}

fn ctx_size(ctx: &web::CanvasRenderingContext2d) -> Option<[u32; 2]> {
    let canvas = ctx.canvas()?;
    Some([canvas.width(), canvas.height()])
}

#[allow(unused)]
pub fn log(value: &str) {
    unsafe { web_sys::console::debug_1(&JsValue::from_str(value)); }
}

#[allow(unused)]
pub fn log_value<D: ::std::fmt::Debug + ?Sized>(value: &D, pretty: bool) {
    match pretty {
        true => log(&format!("{:#?}", value)),
        false => log(&format!("{:?}", value))
    }
}

#[allow(unused)]
pub fn log_js(value: &JsValue) {
    unsafe { web_sys::console::debug_1(value); }
}

pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}
