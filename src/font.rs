use ttf_parser::{Face, FaceParsingError, name_id::FULL_NAME};
use std::{ops::Deref, collections::HashMap};

use super::{font_geometry::{FontGeometry, OutlineBounds}, Line, SdfRaster, sdf_generate};


/// Settings for controlling specific font and layout behavior.
#[derive(Copy, Clone, Debug)]
pub struct FontSettings {
    /// The default is 0. The index of the font to use if parsing a font collection.
    pub collection_index: u32,
}

impl Default for FontSettings {
    fn default() -> Self {
        FontSettings { collection_index: 0 }
    }
}

/// Metrics associated with line positioning.
/// Taken from fontdue
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct LineMetrics {
    /// The highest point that any glyph in the font extends to above the baseline. Typically
    /// positive.
    pub ascent: f32,
    /// The lowest point that any glyph in the font extends to below the baseline. Typically
    /// negative.
    pub descent: f32,
    /// The gap to leave between the descent of one line and the ascent of the next. This is of
    /// course only a guideline given by the font's designers.
    pub line_gap: f32,
    /// A precalculated value for the height or width of the line depending on if the font is laid
    /// out horizontally or vertically. It's calculated by: ascent - descent + line_gap.
    pub new_line_size: f32,
}

impl LineMetrics {
    /// Creates a new line metrics struct and computes the new line size.
    fn new(ascent: i16, descent: i16, line_gap: i16) -> LineMetrics {
        // Operations between this values can exceed i16, so we extend to i32 here.
        let (ascent, descent, line_gap) = (ascent as i32, descent as i32, line_gap as i32);
        LineMetrics {
            ascent: ascent as f32,
            descent: descent as f32,
            line_gap: line_gap as f32,
            new_line_size: (ascent - descent + line_gap) as f32,
        }
    }

    /// Scales the line metrics by the given factor.
    #[inline(always)]
    fn scale(&self, scale: f32) -> LineMetrics {
        LineMetrics {
            ascent: self.ascent * scale,
            descent: self.descent * scale,
            line_gap: self.line_gap * scale,
            new_line_size: self.new_line_size * scale,
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Metrics {
    /// The width of the bitmap in whole pixels.
    pub width: i32,
    /// The height of the bitmap in whole pixels.
    pub height: i32,
    /// Advance width of the glyph in subpixels. Used in horizontal fonts.
    pub advance_width: f32,
    /// The bounding box that contains the glyph's outline at the offsets specified by the font.
    pub bounds: OutlineBounds
}


#[derive(Default)]
pub(crate) struct Glyph {
    pub bounds: OutlineBounds,
    pub advance_width: f32,
    pub lines: Vec<Line>,
}

/// Represents a font. Once loaded, the Font instance can be then used to rasterize its glyphs as sdf.
///
/// Example
/// ```rust
/// use easy_signed_distance_field as sdf;
/// 
/// /// A typical implementation will cache the Font instance
/// fn rasterize_font(font_data: &[u8]) -> (sdf::Metrics, sdf::SdfRaster) {
///     let font = sdf::Font::from_bytes(font_data, sdf::FontSettings::default()).unwrap();
///     let size_px = 64.0;
///     let padding = 2;
///     let spread = 8.0;
///     let (metrics, glyph_sdf) = font.sdf_generate(size_px, padding, spread, 'a').unwrap();
///     (metrics, glyph_sdf)
/// } 
/// ```
/// 
/// Credits to https://github.com/mooman219/fontdue from which most of the font api was ~copied~ inspired
pub struct Font {
    name: Option<String>,
    glyphs: HashMap<char, Glyph>,
    horizontal_line_metrics: LineMetrics,
    units_per_em: f32,
}

impl Font {

    /// Loads a font instance from an array of bytes
    pub fn from_bytes<D: Deref<Target = [u8]>>(data: D, settings: FontSettings) -> Result<Self, FaceParsingError> {
        let face = Face::from_slice(&data, settings.collection_index)?;
        let name = convert_name(&face);
        let units_per_em = face.units_per_em() as f32;

        // Collect all the unique codepoint to glyph mappings.
        let glyph_count = face.number_of_glyphs();
        let mut glyph_id_mapping = HashMap::with_capacity(glyph_count as usize);
        if let Some(subtable) = face.tables().cmap {
            for subtable in subtable.subtables {
                subtable.codepoints(|codepoint| {
                    if let Some(mapping) = subtable.glyph_index(codepoint) {
                        glyph_id_mapping.insert(codepoint, mapping);
                    }
                })
            }
        }
        

        let mut glyphs = HashMap::with_capacity(glyph_id_mapping.len());
        for (codepoint, glyph_id) in glyph_id_mapping {
            let char = match char::from_u32(codepoint) {
                Some(c) => c,
                None => continue
            };

            let mut glyph = Glyph::default();

            let mut geometry = FontGeometry::new();
            face.outline_glyph(glyph_id, &mut geometry);
            geometry.finalize();

            glyph.lines = geometry.lines;
            glyph.advance_width = face.glyph_hor_advance(glyph_id).unwrap_or(0) as f32;
            glyph.bounds = geometry.bounds;

            glyphs.insert(char, glyph);
        } 

        let horizontal_line_metrics = LineMetrics::new(face.ascender(), face.descender(), face.line_gap());

        let font = Font {
            name,
            glyphs,
            units_per_em,
            horizontal_line_metrics
        };

        Ok(font)
    }

    /// Returns the name of the font, or `None` if it could not be found
    pub fn name(&self) -> &Option<String> {
        &self.name
    }

    /// Return the `units_per_em` value of the font
    pub fn units_per_em(&self) -> f32 {
        self.units_per_em
    }

    /// Return the size in px at which the character `c` will be rendered with a height of `height` pixels
    /// 
    /// This works great for if the sdf render will be packed into an atlas, as all glyph will
    /// have a similar rendering quality (even smaller ones) and there is no need to worry about packing characters with different
    /// heights in your algorithm.
    /// 
    /// # Arguments
    /// 
    /// * `c`: The character that will be used
    /// * `height`: The height in pixels that the character should be rendered to.
    /// 
    /// # Return
    /// 
    /// Returns the size in px at which `c` will be rendered with a height of `height` pixels.
    /// Returns `None` if `c` is not available in the font face.
    /// 
    /// # Usage
    /// 
    /// ```rust
    /// use easy_signed_distance_field as sdf;
    /// 
    /// /// Renders a character sdf with a height of 60 pixels
    /// fn render_char_60px(font: &sdf::Font, c: char) -> Option<(sdf::Metrics, sdf::SdfRaster)> {
    ///     let px = font.char_height_to_font_size(c, 60.0)?;
    ///     font.sdf_generate(px, 2, 9.0, c)
    /// }
    /// ```
    pub fn char_height_to_font_size(&self, c: char, height: f32) -> Option<f32> {
        let glyph = self.glyphs.get(&c)?; 
        let base_height = glyph.bounds.height;
        let scale = height / base_height;
        Some(scale * self.units_per_em)
    }

    /// Return the metrics of character `c` scaled to fit a font size of X `px`.
    /// Returns `None` if `c` is not a character in the font face.
    /// # Arguments
    ///
    /// * `px` - The size to scale the glyph metrics by. The units of the scale are pixels per Em unit.
    pub fn metrics(&self, c: char, px: f32) -> Option<Metrics> {
        let scale = self.scale_factor(px);

        let glyph = self.glyphs.get(&c)?;
        let bounds = glyph.bounds.scale(scale);
        let metrics = Metrics {
            width: bounds.width as i32,
            height: bounds.height as i32,
            advance_width: glyph.advance_width * scale,
            bounds: bounds,
        };

        Some(metrics)
    }

    /// New line metrics for fonts that append characters to lines horizontally, and append new
    /// lines vertically (above or below the current line). Only populated for fonts with the
    /// appropriate metrics, none if it's missing.
    /// # Arguments
    ///
    /// * `px` - The size to scale the line metrics by. The units of the scale are pixels per Em unit.
    pub fn horizontal_line_metrics(&self, px: f32) -> LineMetrics {
        let metrics = self.horizontal_line_metrics;
        metrics.scale(self.scale_factor(px))
    }

    /// Generates the sdf for the character `c`. The font instance scale will be used for the output size.
    /// Use [sdf_generate] under the hood.
    /// 
    /// # Arguments
    ///
    /// * `px` - The size to scale the glyph by. Th e units of the scale are pixels per Em unit.
    /// * `padding` - Padding (in px) to add around the glyph. Should be > 0
    /// * `spread` - Control how the gradient in the sdf spread. 
    /// * `c` - Character to render
    /// 
    /// # Return
    /// 
    /// Returns Ok(Some([Metrics], [SdfRaster])) if the render was successful
    /// 
    /// Returns `Ok(None)` if `c` is not a character in the font face.
    /// 
    /// Returns `Err(_)` if the sdf generation failed
    /// 
    /// # Panics
    /// 
    /// Panics if `px` is smaller than 1.0
    pub fn sdf_generate(&self, px: f32, padding: i32, spread: f32, c: char) -> Option<(Metrics, SdfRaster)> {
        if px < 1.0 {
            panic!("Sdf render size cannot be smaller than 1.0 (got {:?})", px);
        }

        let glyph = match self.glyphs.get(&c) {
            Some(g) => g,
            None => { return None; }
        };

        let metrics = self.metrics(c, px).unwrap(); // Cannot return `None` if glyph is some

        //println!("{:?} {:?}", metrics.width, metrics.height);

        let sdf = sdf_generate(metrics.width as u32, metrics.height as u32, padding, spread, &glyph.lines);

        Some((metrics, sdf))
    }


    pub fn lines(&self, g: char) {
        let glyph = self.glyphs.get(&g).unwrap();
        for line in glyph.lines.iter() {
            println!("{:?}", line);
        }
    }

    fn scale_factor(&self, px: f32) -> f32 {
        px / self.units_per_em
    }

}


fn convert_name(face: &Face) -> Option<String> {
    for name in face.names() {
        if name.name_id == FULL_NAME && name.is_unicode() {
            return name.to_string();
        }
    }
    None
}
