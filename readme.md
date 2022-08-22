# Easy signed distance field

Easy signed distance field is a simple, pure rust signed distance field renderer. It is meant to be used as a simple, more versatile, more feature complete, alternative to the other sdf crates available on crates.io. 

Easy signed distance field currently works on raw line inputs and ttf/otf font inputs. It also supports CPU rendering for debugging purpose.

Signed distance field is a rendering technique that can generate very small glyphs that can be rendered into much higher resolutions at the cost of losing quality around sharp corners. The technique also comes with "free" anti aliasing and some text effects.

Generally, this technique is used in games to create altases of small character glyphs that can then be rendered at any resolution. Rendering is done on a gpu on a fragment shader.

For a live demo, head over to https://gabdube.github.io/easy-signed-distance-field/

For the documentation see https://docs.rs/easy-signed-distance-field/0.1.1/easy_signed_distance_field/

![sdf render](images/font_a_render.png) ![sdf](images/font_a.png)

# Features

By default, this library is dependency free and only supports rendering from a collection of lines.
Extra features can be enabled

* `font`: Import font family and render character glyph as sdf
* `export`: Export sdf as image
* `render`: (**with export**) renders an sdf to a file

```toml
[features]
default = []
render = []
font = ["ttf-parser"]
export = ["image"]
```

# Usage

Rendering a simple triangle from a collection of lines

```rust
use easy_signed_distance_field as sdf;

let lines = [
    Line::Line { start: vec2(0.5, 0.0), end: vec2(1.0, 1.0) },
    Line::Line { start: vec2(1.0, 1.0), end: vec2(0.0, 1.0) },
    Line::Line { start: vec2(0.0, 1.0), end: vec2(0.5, 0.0) },
];

let width: u32 = 32;
let height: u32 = 32;
let padding = 2;
let spread = 5.0;
let sdf = sdf::sdf_generate(
    width,
    height,
    padding,
    spread,
    &lines,
);

#[cfg(feature="export")]
sdf::sdf_to_file("test_outputs/triangle.png", &sdf).unwrap();


let render_scale = 512.0 / (size as f32);

#[cfg(feature="render")]
#[cfg(feature="export")]
sdf::sdf_render_to_file("test_outputs/triangle_render.png", render_scale, 0.5, 0.02, &sdf).unwrap();
```

Rendering a character from a font

```rust
use std::fs;
use easy_signed_distance_field as sdf;

let font_data = fs::read("./test_fixtures/Questrial-Regular.ttf").expect("Failed to read font file");
let font = sdf::Font::from_bytes(font_data.as_slice(), Default::default()).expect("Failed to parse font file");

let px = 64.0;
let padding = 2;
let spread = 6.0;
let (a_metrics, a_glyph_sdf) = font.sdf_generate(px, padding, spread, 'a').unwrap();

#[cfg(feature="export")]
sdf::sdf_to_file("test_outputs/font_a.png", &a_glyph_sdf).unwrap();

#[cfg(feature="render")]
#[cfg(feature="export")]
sdf::sdf_render_to_file("test_outputs/font_a_render.png", render_scale, 0.5, 0.02, &a_glyph_sdf).unwrap();
```

Uploading a sdf to a webgl texture

```rust
use std::fs;
use easy_signed_distance_field as sdf;

use web_sys as web;
use web_sys::WebGl2RenderingContext as GL;

let font_data = fs::read("./test_fixtures/Questrial-Regular.ttf").expect("Failed to read font file");
let font = sdf::Font::from_bytes(font_data.as_slice(), Default::default()).expect("Failed to parse font file")

let px = 64.0;
let padding = 2;
let spread = 6.0;
let (a_metrics, a_glyph_sdf) = font.sdf_generate(px, padding, spread, 'a').unwrap();

let sdf_bitmap = sdf::sdf_to_bitmap(sdf);

let ctx: web::WebGl2RenderingContext = get_context(); // Pseudo code that fetches your webgl context

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

```


# Performances

According to the benchmarks, generating the sdf of a single character at 64px takes around 2ms. Generating the whole alphabet a the same resolution, both in lower case and upper case, takes around 66 ms. This timing will change depending on how complicated your font/shapes are.

This means if you are planning to use this library in a real time environment (ex: generating sdf of characters as the user type), it's better to run the code off the main thread OR pre-generate the characters at startup.

Still, the algorithm is single threaded and very naive, and there are probably plenty of ways to improve it.

# Roadmap

This project was developed to be used into one of my pet projects, as such further development/maintenance will only be done if that project requires it. That said I won't mind merging features done by contributors, just make sure to discuss it with me in the issues sections if it's something big.

Implementing multi signed distance field (msdf) is also something I want to add at some point.

Adding multithreading support is also something I want to explore. Although, this is not a priority as I'm planning to use this from wasm.

This project won't reach `1.0` until I (or somebody else) use it in a serious project. Until then, the API is subject to change.

# License

This code is MIT licensed.

# Attributions

While I'm not **that** bad at math, I am not a mathematician; most of the algorithm used in this project were created by people much more competent than me.

* https://www.shadertoy.com/view/MlKcDD for the quad bezier distance function
* https://pomax.github.io/bezierinfo/#introduction for the overall theory behind bezier curves
* MsdfGen https://github.com/Chlumsky/msdfgen as the inspiration for this project
* Fontdue: https://github.com/mooman219/fontdue , which was used as a template to design the font interface
* bezierjs https://github.com/Pomax/bezierjs
