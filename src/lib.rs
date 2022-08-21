mod math;
pub use math::{vec2, Vec2};

mod line;
pub use line::Line;

#[cfg(feature="path")]
pub(crate) mod path;
#[cfg(feature="path")]
pub use path::*;

#[cfg(feature="font")]
pub(crate) mod font_geometry;

#[cfg(feature="font")]
mod font;
#[cfg(feature="font")]
pub use font::*;

/// SDF output of a shape by [sdf_raster]
pub struct SdfRaster {
    /// Width of the buffer in pixel
    pub width: u32,
    /// Height of the buffer in pixel
    pub height: u32,
    /// Buffer data. Each values represent the distance of the pixel to the nearest line
    /// Values range from 0.0 (outside) to 1.0 (inside) with 0.5 being directly on a line
    pub buffer: Vec<f32>
}

/// SDF output of a shape converted to an array of bytes using [sdf_to_bitmap].
pub struct SdfBitmap {
    /// Width of the buffer in pixel
    pub width: u32,
    /// Height of the buffer in pixel
    pub height: u32,
    /// Buffer data. Each values represent the distance of the pixel to the nearest line
    /// Values range from 0 (outside) to 255 (inside) with 127 being directly on a line
    pub buffer: Vec<u8>
}

/// Rasterize a shape defined by `lines`
/// # Arguments
/// 
/// * `width`: Width (in pixels) of the output raster
/// * `height`: Height (in pixels) of the output raster
/// * `padding`: Padding added to the output raster, this won't change the bitmap size, but it will reduce the sdf quality as the work area will be smaller
/// * `spread`: Control how the gradient in the sdf spread. A higher value means less spread. 15.0 is a good default value
/// For best result the outer borders of your sdf should be pure black and the inner part of the shape should be pure white.
/// * `lines`: List of [line::Line] to be rasterized. Assumes the shape is closed and that the lines coordinates are normalized (aka between 0.0 and 1.0).
/// 
/// # Note
/// 
/// * The padding is added to the final image size. Ie: `width+(2*padding)` X `height+(2*padding)`. 
/// * Padding must be added to shapes with lines near the edges of the raster (`0.0` or `1.0`), otherwise those edge will be clipped in the sdf
///  You can (and should) skip the padding if all the edges are already far enough from the borders.
pub fn sdf_generate(
    width: u32,
    height: u32,
    padding: i32,
    spread: f32,
    lines: &[line::Line]
) -> SdfRaster {
    let mut lines = lines;
    let mut padded_lines: Vec<line::Line> = Vec::with_capacity(lines.len());
    if padding != 0 {
        let padding_width_normalized = padding as f32 / width as f32;
        let padding_height_normalized = padding as f32 / height as f32;
        for line in lines.iter() {
            padded_lines.push(line.normalize_to_with_offset(
                -padding_width_normalized,
                -padding_height_normalized,
                1.0 as f32 + (padding_width_normalized * 2.0),
                1.0 as f32 + (padding_height_normalized * 2.0)
            ));
        }

        lines = padded_lines.as_slice();
    }

    let _1w = 1.0 / width as f32;
    let _1h = 1.0 / height as f32;
    
    let buffer_size = (width * height) as usize;
    let mut image_buffer: Vec<f32> = vec![0.0; buffer_size];
    
    // Compute the distance between lines
    for x in 0..width {
        for y in 0..height {
            let px = (x as f32 + 0.5) * _1w;
            let py = (y as f32 + 0.5) * _1h;
            let index = (x + (width * y)) as usize;

            let mut min_distance = f32::MAX;
            for line in lines {
                let d = line.distance(px, py);
                if d < min_distance {
                    min_distance = d;
                }
            }

            min_distance = (1.0 - (min_distance * spread)) - 0.5;
            image_buffer[index] = min_distance.clamp(0.0, 1.0);
        }
    }

    // Flip if a pixel is inside or outside the shape
    for y in 0..height {
        let py = (y as f32 + 0.5) * _1h;
        let scanline = scanline(py, lines);
        for x in 0..width {
            let index = (x + (width * y)) as usize;
            let px = (x as f32 + 0.5) * _1w;
            if scanline_scan(&scanline, px) {
                image_buffer[index] = 1.0 - image_buffer[index];
            }
        }
    }

    SdfRaster {
        width: width,
        height: height,
        buffer: image_buffer,
    }
}

/// Convert and [SdfRaster] into a [SdfBitmap].
/// A bitmap is usually what to you to send to store in a gpu texture.
/// 
/// # Arguments
/// 
/// * sdf: The sdf to convert
pub fn sdf_to_bitmap(sdf: &SdfRaster) -> SdfBitmap {
    let width = sdf.width;
    let height = sdf.height;
    let mut buffer: Vec<u8> = vec![0u8; (width * height) as usize];

    for x in 0..width {
        for y in 0..height {
            let index = (x + (width * y)) as usize;
            buffer[index] = (sdf.buffer[index] * 255.0) as u8;
        }
    }

    SdfBitmap { width, height, buffer }
}

/// Saves a sdf output to a file. 
/// # Arguments
/// 
/// * `output_name`: Name of the file to output the sdf buffer
/// * `sdf`: [SdfRaster] to output
/// 
/// # Return
/// 
/// * Returns the result of the `image.save` function
/// 
/// # Note
///   * Png (or other lossless format) are strongly recommended.
///   * The file format will use a single byte grayscale pixel format.
/// 
/// # Feature
/// 
/// * Requires the `export` feature with the used file format subfeature (ex: png, jpeg, etc)
#[cfg(feature="export")]
pub fn sdf_to_file(output_name: &str, sdf: &SdfRaster) -> image::ImageResult<()> {
    use image::{GrayImage, Luma};

    let width = sdf.width;
    let height = sdf.height;

    let mut img = GrayImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let index = (x + (width * y)) as usize;
            let pixel_value = (sdf.buffer[index] * 255.0) as u8;
            img.put_pixel(x, y, Luma([pixel_value]));
        }
    }

    img.save(output_name)
}
 

/// Renders an image from an SDF buffer previously generated by `sdf_raster`, scaled by `scale`. 
/// 
/// Arguments
/// 
/// * `output_name`: Output name for the sdf render
/// * `output_scale`: Scale of the output image. Ex: a 16x16 sdf raster with 4x scale will have a final size of `64x64`. 
///   You usually want a scaling > 1.0, but downscaling is also supported
/// * `mid_value`: The distance value at which to cut between a empty pixel and a filled one. You usually want `~0.5` for this.
/// * `smoothing`: Shape edge smoothing. Think of this as cheap anti aliasing. Disabled if set to `0.0`. Should be between `0.0` and `0.05`
/// * `sdf`: The [SdfRaster] to render
/// 
/// # Return
/// 
/// Returns the result of the `image.save` function
/// 
/// # Panic
/// 
/// * Panics if scale <= 0.0 or if the scale value returns an image with zeroed dimensions
/// * May panic if the image is too large and the memory allocation fails
/// 
/// # Note
/// 
/// * The output file format will use a single byte grayscale format
/// 
/// # Feature
/// 
/// * Require the `export` feature with the used file format subfeature (ex: png, jpeg, etc)
/// * Requires the `render` feature
#[cfg(feature="export")]
#[cfg(feature="render")]
pub fn sdf_render_to_file(
    output_name: &str,
    scale: f32,
    mid_value: f32,
    smoothing: f32,
    sdf: &SdfRaster,
) -> image::ImageResult<()> {
    use image::{GrayImage, Luma};

    let width = (sdf.width as f32 * scale) as u32;
    let height = (sdf.height as f32 * scale) as u32;
    if width <= 0 || height <= 0 {
        panic!("Scaling of {:?} returns an image size of {:?}, which is impossible to render", scale, (width, height));
    }

    let width_f = width as f32;
    let height_f = height as f32;

    let distance_to_pixel = |distance: f32| {
        match distance > mid_value {
            true => (smoothstep(mid_value-smoothing, mid_value+smoothing, distance) * 255.0) as u8,
            false => 0,
        }
    };

    let mut img = GrayImage::new(width, height);
    for x in 0..width {
        for y in 0..height {
            let sample_x = ((x as f32) + 0.5) / width_f;
            let sample_y = ((y as f32) + 0.5) / height_f;

            let sampled_distance = sdf_sample(sdf, sample_x, sample_y);
            let pixel_value = distance_to_pixel(sampled_distance);
            img.put_pixel(x, y, Luma([pixel_value]));
        }
    }
   
    img.save(output_name) 
}

/// Samples a pixel value in `sdf` at (`x`, `y`). `x` and `y` being normalized coordinates between `0.0` and `1.0`
/// 
/// # Arguments
/// 
/// * `sdf` The sdf to sample
/// * `x`: The normalized x coordinate to sample
/// * `y`: The normalized y coordinate to sample
/// 
/// Return
/// 
/// The interpolated distance between the 4 nearest pixels
/// 
/// Returns the sampled distance. Guaranteed to be between 0.0 and 1.0
pub fn sdf_sample(sdf: &SdfRaster, x: f32, y: f32) -> f32 {
    let gx = (x * (sdf.width as f32) - 0.5).max(0.0);
    let gy = (y * (sdf.height as f32) - 0.5).max(0.0);
    let left = gx.floor() as usize;
    let top = gy.floor() as usize;
    let wx = gx - (left as f32);
    let wy = gy - (top as f32);

    let right = (left+1).min((sdf.width - 1) as usize);
    let bottom = (top+1).min((sdf.height - 1) as usize);

    let row_size = sdf.width as usize;   
    let get_pixel = |x, y| {
        sdf.buffer[(row_size*y)+x]
    };

    let p00 = get_pixel(left, top);
    let p10 = get_pixel(right, top);
    let p01 = get_pixel(left, bottom);
    let p11 = get_pixel(right, bottom);

    mix(mix(p00, p10, wx), mix(p01, p11, wx), wy)
}

/// Collection of intersection between an horizontal line and multiple other lines.
struct Scanline {
    intersections: Vec<f32>,
}

/// Scan all the intersection for an horizontal line at `y`
fn scanline(y: f32, lines: &[line::Line]) -> Scanline {
    let mut scanline = Scanline { intersections: Vec::with_capacity(16) };
    let mut x = [0.0, 0.0, 0.0];

    for line in lines {
        let count = line.intersections(y, &mut x);
        for i in 0..count {
            scanline.intersections.push(x[i]);
        }
    }

    if scanline.intersections.len() > 0 {
        scanline.intersections.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    }
    
    scanline
}

/// Check if a point `x` in inside or outside `scanline`
fn scanline_scan(scanline: &Scanline, x: f32) -> bool {
    let count = scanline
        .intersections
        .iter()
        .fold(0u32, |acc, &inter| match x < inter {
            true => acc+1,
            false => acc
        });

    count % 2 == 1
}

/// Linear interpolation function
fn mix(v1: f32, v2: f32, weight: f32) -> f32 {
    v1 + (v2-v1) * weight
}

/// GLSL-like smoothstep function
/// perform Hermite interpolation between two values
#[cfg(feature="render")]
fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x-edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}


#[cfg(test)]
mod tests {
    #![allow(unused_variables)]
    use super::math::vec2;
    use super::line::Line;
    use super::*;

    #[test]
    fn test_distance_functions() {
        // Linear
        let line = Line::Line { start: vec2(0.0, 0.0), end: vec2(1.0, 1.0) };
        assert_eq!(line.distance(0.0, 0.0), 0.0);  
        assert_eq!(line.distance(1.0, 1.0), 0.0);  

        let dx = 1.0f32 - 0.5;
        let dy = 0.0f32 - 0.5;
        let d = ((dx*dx) + (dy*dy)).sqrt();
        assert_eq!(line.distance(1.0, 0.0), d);  

        // Quadratic
        let line = Line::Quad { start: vec2(0.0, 0.0), end: vec2(1.0, 1.0), control: vec2(1.0, 0.0) };
        assert_eq!(line.distance(0.0, 0.0), 0.0);  
        assert_eq!(line.distance(1.0, 1.0), 0.0);
        assert_eq!(line.distance(1.0, 0.0), 0.3535533905932738);
        assert_eq!(line.distance(0.0, 1.0), 1.0);
        assert_eq!(line.distance(0.5, 1.0), 0.5);
        assert_eq!(line.distance(0.0, 0.5), 0.5);

        // Cubic
        let line = Line::Curve { start: vec2(0.0, 0.0), end: vec2(1.0, 1.0), first_control: vec2(0.8, 0.0), second_control: vec2(1.0, 0.2) };
        assert_eq!(line.distance(0.0, 0.0), 0.0);  
        assert_eq!(line.distance(1.0, 1.0), 0.0);

        let d0 = line.distance(1.0, 0.0);
        assert!(d0 > 0.28284 && d0 < 0.28285);

        let d1 = line.distance(0.8, 0.0);
        //assert!(d1 > 0.15718 && d1 < 0.15719);

        let d2 = line.distance(0.8, 0.0);
        //assert!(d2 > 0.15718 && d2 < 0.15719);
    }

    #[test]
    fn test_intersection() {
        let intersection_1 = |line: &Line, y: f32| {
            let mut out = Default::default();
            let count = line.intersections(y, &mut out);
            assert_eq!(count, 1, "y: {:?} line: {:?} out: {:?}", y, line, out);
            out[0]
        };

        let intersection_2 = |line: &Line, y: f32| {
            let mut out = Default::default();
            let count = line.intersections(y, &mut out);
            assert_eq!(count, 2, "y: {:?} line: {:?}", y, line);
            [out[0], out[1]]
        };

        let intersection_3 = |line: &Line, y: f32| {
            let mut out = Default::default();
            let count = line.intersections(y, &mut out);
            assert_eq!(count, 3, "y: {:?} line: {:?}", y, line);
            out
        };
        

        // Linear
        let line = Line::Line { start: vec2(0.0, 0.0), end: vec2(1.0, 1.0) };
        assert_eq!(intersection_1(&line, 0.0), 0.0);
        assert_eq!(intersection_1(&line, 0.5), 0.5);
        assert_eq!(intersection_1(&line, 1.0), 1.0);
        assert_eq!(line.intersections(1.5, &mut Default::default()), 0);
        assert_eq!(line.intersections(-0.5, &mut Default::default()), 0);

        // Quad
        let line = Line::Quad { start: vec2(0.0, 0.0), end: vec2(1.0, 1.0), control: vec2(1.0, 0.0) };
        assert_eq!(intersection_1(&line, 0.0), 0.0);
        assert_eq!(intersection_1(&line, 1.0), 1.0);
        assert_eq!(line.intersections(1.5, &mut Default::default()), 0);
        assert_eq!(line.intersections(-0.5, &mut Default::default()), 0);

        let d0 = intersection_1(&line, 0.1);
        assert!(d0 > 0.532455 && d0 < 0.532456, "{}", d0);

        let d1 = intersection_1(&line, 0.5);
        assert!(d1 > 0.914213 && d1 < 0.914214, "{}", d1);

        let d2 = intersection_1(&line, 0.9);
        assert!(d2 > 0.997366 && d2 < 0.997367, "{}", d2);

        let line = Line::Quad { start: vec2(0.0, 1.0), end: vec2(1.0, 1.0), control: vec2(0.5, 0.0) };
        assert_eq!(line.intersections(0.0, &mut Default::default()), 0);

        let [d3, d4] = intersection_2(&line, 0.9);
        assert!(d3 > 0.947213 && d3 < 0.947214, "{}", d3);
        assert!(d4 > 0.052786 && d4 < 0.052787, "{}", d4);

        // Cubic
        /*
        let line = Line::Curve { start: vec2(0.0, 0.0), end: vec2(1.0, 1.0), first_control: vec2(0.8, 0.0), second_control: vec2(1.0, 0.2) };
        
        let d0 = intersection_1(&line, 0.0);
        assert!(d0 >= 0.0 && d0 < 0.0004, "{}", d0);

        let d1 = intersection_1(&line, 1.0);
        assert!(d1 >= 0.9999 && d1 <= 1.0, "{}", d1);

        let d2 = intersection_1(&line, 0.5);
        assert!(d2 > 0.954741 && d2 < 0.954742, "{}", d2);

        let line = Line::Curve { start: vec2(0.0, 1.0), end: vec2(1.0, 1.0), first_control: vec2(0.4, 0.0), second_control: vec2(0.6, 0.0) };
        assert_eq!(line.intersections(0.0, &mut Default::default()), 0);

        let [d3, d4] = intersection_2(&line, 0.7);
        assert!(d3 > 0.871806 && d3 < 0.871807, "{}", d3);
        assert!(d4 > 0.112701 && d4 < 0.112702, "{}", d4);

         */
    }

    #[test]
    fn test_triangle() {
        let lines = [
            Line::Line { start: vec2(0.5, 0.0), end: vec2(1.0, 1.0) },
            Line::Line { start: vec2(1.0, 1.0), end: vec2(0.0, 1.0) },
            Line::Line { start: vec2(0.0, 1.0), end: vec2(0.5, 0.0) },
        ];

        let size: u32 = 32;
        let padding = 2;
        let render_scale = 512.0 / (size as f32);

        let sdf = sdf_generate(
            size,
            size,
            padding,
            5.0,
            &lines,
        );

        let sdf_bin = sdf_to_bitmap(&sdf);

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/triangle.png", &sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/triangle_render.png", render_scale, 0.5, 0.02, &sdf).unwrap();
    }

    #[test]
    fn test_box_hole() {
        let lines = [
            Line::Line { start: vec2(0.0, 0.0), end: vec2(1.0, 0.0) },
            Line::Line { start: vec2(1.0, 0.0), end: vec2(1.0, 1.0) },
            Line::Line { start: vec2(1.0, 1.0), end: vec2(0.0, 1.0) },
            Line::Line { start: vec2(0.0, 1.0), end: vec2(0.0, 0.0) },

            Line::Line { start: vec2(0.5, 0.25), end: vec2(0.25, 0.75) },
            Line::Line { start: vec2(0.25, 0.75), end: vec2(0.75, 0.75) },
            Line::Line { start: vec2(0.75, 0.75), end: vec2(0.5, 0.25) },
        ];

        let size = 32;
        let padding = 3;
        let render_scale = 512.0 / (size as f32);

        let sdf = sdf_generate(
            size,
            size,
            padding,
            8.0,
            &lines
        );

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/box_hole.png", &sdf).unwrap();
       
        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/box_hole_render.png", render_scale, 0.5, 0.02, &sdf).unwrap();
        
        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/box_hole_render_downscale.png", 0.5, 0.5, 0.02, &sdf).unwrap();
    }

    #[test]
    fn test_quad_bezier() {
        let view_box_width = 24.0;
        let view_box_height = 24.0;
        let mut lines = [
            Line::Quad { start: vec2(4.0, 10.0), end: vec2(20.0, 15.0), control: vec2(12.0, 0.0) },
            Line::Quad { start: vec2(20.0, 15.0), end: vec2(4.0, 10.0), control: vec2(12.0, 24.0) },
        ];

        // Normalize the lines values between 0.0 and 1.0
        for line in lines.iter_mut() {
            line.normalize(view_box_width, view_box_height);
        }

        let size = 32;
        let render_scale = 512.0 / (size as f32);

        let sdf = sdf_generate(
            size,
            size,
            0,
            5.0,
            &lines
        );

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/quad_bezier.png", &sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/quad_bezier_render.png", render_scale, 0.5, 0.02, &sdf).unwrap();
    }

    #[test]
    fn test_cubic_bezier() {
        // A flag-like shape
        let lines = [
            Line::Curve { start: vec2(0.1, 0.9), end: vec2(0.9, 0.9), first_control: vec2(0.4, 0.5), second_control: vec2(0.6, 1.1) },
            Line::Line { start: vec2(0.9, 0.9), end: vec2(0.9, 0.1) },
            Line::Curve { start: vec2(0.9, 0.1), end: vec2(0.1, 0.1), first_control: vec2(0.6, 0.5), second_control: vec2(0.2, -0.1) },     
            Line::Line { start: vec2(0.1, 0.1), end: vec2(0.1, 0.9) },
        ];

        let size = 32;
        let render_scale = 512.0 / (size as f32);

        let sdf = sdf_generate(
            size,
            size,
            0,
            5.0,
            &lines
        );

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/cubic_bezier.png", &sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/cubic_bezier_render.png", render_scale, 0.5, 0.01, &sdf).unwrap();
    }

    #[test]
    fn test_stretch() {
        let lines = [
            Line::Line { start: vec2(0.1, 0.1), end: vec2(0.9, 0.1) },
            Line::Line { start: vec2(0.9, 0.1), end: vec2(0.9, 0.9) },
            Line::Line { start: vec2(0.9, 0.9), end: vec2(0.1, 0.9) },
            Line::Line { start: vec2(0.1, 0.9), end: vec2(0.1, 0.1) },
        ];

        let width = 64;
        let height = 32;
        let padding = 0;
        let render_scale = 512.0 / (width as f32);

        let sdf = sdf_generate(
            width,
            height,
            padding,
            8.0,
            &lines
        );

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/stretch.png", &sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/stretch_render.png", render_scale, 0.5, 0.02, &sdf).unwrap();
    }

    #[cfg(feature="font")]
    #[test]
    fn test_font() {
        use std::fs;

        let font_data = fs::read("./test_fixtures/Questrial-Regular.ttf").expect("Failed to read font file");
        let font = Font::from_bytes(font_data.as_slice(), Default::default()).expect("Failed to parse font file");

        assert!(font.name().as_ref().map(|v| v.as_str()) == Some("Questrial Regular"));
        assert!(font.units_per_em() == 1000.0);

        let px = 100.0;
        let (a_metrics, a_glyph_sdf) = font.sdf_generate(px, 2, 8.0, 'a').unwrap();
        let (y_metrics, y_glyph_sdf) = font.sdf_generate(px, 2, 8.0, 'y').unwrap();
        let (i_metrics, i_glyph_sdf) = font.sdf_generate(px, 2, 8.0, 'i').unwrap();
        
        let render_scale = 512.0 / px;
        
        #[cfg(feature="export")]
        sdf_to_file("test_outputs/font_a.png", &a_glyph_sdf).unwrap();

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/font_y.png", &y_glyph_sdf).unwrap();

        #[cfg(feature="export")]
        sdf_to_file("test_outputs/font_i.png", &i_glyph_sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/font_a_render.png", render_scale, 0.5, 0.02, &a_glyph_sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/font_y_render.png", render_scale, 0.5, 0.02, &y_glyph_sdf).unwrap();

        #[cfg(feature="render")]
        #[cfg(feature="export")]
        sdf_render_to_file("test_outputs/font_i_render.png", render_scale, 0.5, 0.02, &i_glyph_sdf).unwrap();
    }

    #[cfg(feature="font")]
    #[test]
    fn test_font_fixed_height() {
        use std::fs;

        let font_data = fs::read("./test_fixtures/Questrial-Regular.ttf").expect("Failed to read font file");
        let font = Font::from_bytes(font_data.as_slice(), Default::default()).expect("Failed to parse font file");

        let px = font.char_height_to_font_size('a', 100.0).unwrap();
        let (metrics, glyph_sdf) = font.sdf_generate(px, 0, 8.0, 'a').unwrap();
        assert_eq!(metrics.height, 100);
    }

    // #[cfg(feature="path")]
    // #[test]
    // fn test_path() {
    //     // From https://www.svgviewer.dev/s/13417/pet-14
    //     let path = "
    //         M 9.412959,0.00215164 C 8.1693111,0.14587964 7.4631991,1.5093558 7.3733471,2.6936522 7.2264421,3.7464942 7.6860601,5.0899481 8.7918901,5.3320382 9.750208,5.4175142 10.512633,4.5171926 10.825819,3.6583651 11.195679,2.5833906 11.206482,1.147845 10.309038,0.34250684 10.059753,0.12647084 9.739671,-0.00476426 9.412959,0.00215164 z 
    //         m -5.0729339,0.056367 c -1.034807,0.074858 -1.5652,1.27744256 -1.493223,2.24276386 0.0088,1.2907239 0.717612,2.7424122 2.010186,3.0488129 0.963956,0.1042448 1.606661,-0.9181087 1.66656,-1.812036 0.123011,-1.3680543 -0.483132,-3.02530546 -1.855765,-3.43834986 -0.106687,-0.029465 -0.217332,-0.044089 -0.327758,-0.041191 z 
    //         M 12.655043,4.2036246 c -1.388365,0.1017608 -2.266355,1.5813816 -2.326691,2.9184062 -0.07305,0.8707439 0.489487,1.9657608 1.436683,1.9057498 C 13.181597,8.8776256 14.035636,7.298817 14.00043,5.9270567 13.99133,5.1373276 13.490879,4.2365389 12.655043,4.2036246 z
    //         M 0.01075614,6.0085263 c -0.12855096,1.481506 0.908149,3.1560203 2.41413596,3.2493013 0.912778,-0.06096 1.343188,-1.121093 1.238576,-1.9598358 C 3.5801081,6.0604403 2.7526501,4.7931558 1.6775691,4.6275979 0.60248914,4.4620399 -0.00628982,5.221568 0.01075614,6.0085263 z
    //         M 7.1593651,7.2723472 c -1.411887,0.00329 -2.70277,0.9264694 -3.392913,2.1755314 -0.578235,0.9334254 -1.050947,1.9901564 -1.140853,3.1053424 0.01967,0.860048 0.840276,1.575842 1.663122,1.404534 1.173683,-0.152345 2.222485,-0.967418 3.44297,-0.8147 1.014283,0.08097 1.8507729,0.867482 2.8748439,0.856051 0.994345,0.03734 1.56796,-1.102139 1.188753,-1.994632 C 11.387234,10.726579 10.596107,9.6151936 9.746977,8.6142466 9.069618,7.87643 8.1824521,7.2396817 7.1593651,7.2723472 z
    //     ";

    // }

}
