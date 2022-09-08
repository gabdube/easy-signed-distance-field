use ttf_parser::OutlineBuilder;
use super::{Line, math::vec2};

#[derive(Default, Copy, Clone)]
struct Point {
    x: f32,
    y: f32,
}

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        self.x.to_bits() == other.x.to_bits() && self.y.to_bits() == other.y.to_bits()
    }
}

/// Defines the bounds for a glyph's outline in subpixels. A glyph's outline is always contained in
/// its bitmap.
#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub struct OutlineBounds {
    /// Subpixel offset of the left-most edge of the glyph's outline.
    pub xmin: f32,
    /// Subpixel offset of the bottom-most edge of the glyph's outline.
    pub ymin: f32,
    /// The width of the outline in subpixels.
    pub width: f32,
    /// The height of the outline in subpixels.
    pub height: f32,
}

impl OutlineBounds {
    /// Scales the bounding box by the given factor.
    #[inline(always)]
    pub fn scale(&self, scale: f32) -> OutlineBounds {
        OutlineBounds {
            xmin: self.xmin * scale,
            ymin: self.ymin * scale,
            width: self.width * scale,
            height: self.height * scale,
        }
    }
}

/// Utility to build an array of lines from a font glyph
pub(crate) struct FontGeometry {
    start_point: Point,
    previous_point: Point,
    pub bounds: OutlineBounds,
    pub lines: Vec<Line>
}

impl FontGeometry {

    pub fn new() -> FontGeometry {
        FontGeometry {
            start_point: Point::default(),
            previous_point: Point::default(),
            bounds: Default::default(),
            lines: Vec::with_capacity(32),
        }
    }

    pub fn finalize(&mut self) {
        // Compute bounds
        let mut xmin = f32::INFINITY;
        let mut xmax = f32::NEG_INFINITY;
        let mut ymin = f32::INFINITY;
        let mut ymax = f32::NEG_INFINITY;
        for line in self.lines.iter() {
            let [x0, y0, x1, y1] = match line {
                Line::Line { start, end } | Line::Quad { start, end, .. } | Line::Curve { start, end, ..} => {
                    [start[0], start[1], end[0], end[1]]
                }
            };

            if x0 < xmin { xmin = x0; }
            if x1 < xmin { xmin = x1; }
            if x0 > xmax { xmax = x0; }
            if x1 > xmax { xmax = x1; }

            if y0 < ymin { ymin = y0; }
            if y1 < ymin { ymin = y1; }
            if y0 > ymax { ymax = y0; }
            if y1 > ymax { ymax = y1; }
        }

        if xmin == f32::INFINITY || ymin == f32::INFINITY {
            self.bounds = OutlineBounds {
                xmin: 0.0,
                ymin: 0.0,
                width: 0.0,
                height: 0.0,
            };
        } else {
            self.bounds = OutlineBounds {
                xmin,
                ymin,
                width: xmax - xmin,
                height: ymax - ymin,
            };
        }
       
        // Normalize lines
        let b = self.bounds;
        for line in self.lines.iter_mut() {
            line.normalize_with_offset(b.xmin, b.ymin, b.width, b.height);
            line.flip_y();
        }

        // Strip extra memory from lines vec
        self.lines.shrink_to_fit();
    }

}

impl OutlineBuilder for FontGeometry {

    fn move_to(&mut self, x: f32, y: f32) {
        let next_point = Point { x, y };
        self.start_point = next_point;
        self.previous_point = next_point;
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let p1 = self.previous_point;
        self.lines.push(Line::Line { start: vec2(p1.x, p1.y), end: vec2(x, y) });
        self.previous_point = Point { x, y };
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let p1 = self.previous_point;
        self.lines.push(Line::Quad { start: vec2(p1.x, p1.y), end: vec2(x, y), control: vec2(x1, y1) });
        self.previous_point = Point { x, y };
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let p1 = self.previous_point;
        self.lines.push(Line::Curve { 
            start: vec2(p1.x, p1.y),
            end: vec2(x, y),
            first_control: vec2(x1, y1),
            second_control: vec2(x2, y2) 
        });
        self.previous_point = Point { x, y };
    }

    fn close(&mut self) {
        if self.start_point != self.previous_point {
            let p1 = self.previous_point;
            let p2 = self.start_point;
            self.lines.push(Line::Line { start: vec2(p1.x, p1.y), end: vec2(p2.x, p2.y) });
        }
        self.previous_point = self.start_point;
    }

}


