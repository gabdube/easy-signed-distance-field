use crate::math::{Point, Vec2, vec2, vec3};
use crate::mix;

#[derive(Copy, Clone, Debug)]
pub enum Line {
    Line { start: Point, end: Point },
    Quad { start: Point, end: Point, control: Point },
    Curve { start: Point, end: Point, first_control: Point, second_control: Point }
}

impl Line {

    /// Return the distance  of the point [x,y] from the line, where 0 is the right on the line
    pub fn distance(&self, x: f32, y: f32) -> f32 {
        let p = vec2(x, y);
        match *self {
            Self::Line { start, end } => {
                let pa = p - start;
                let ba = end - start;
                let h = (pa.dot(ba) / ba.dot(ba)).clamp(0.0, 1.0);
                (pa - (ba*h)).length().abs()
            },
            Self::Quad { start, end, control } => {
                // Credits to https://www.shadertoy.com/view/MlKcDD
                let pa = control - start;
                let pb = start - control * 2.0 + end;
                let pc = pa * 2.0;
                let pd = start - p;

                let kk = 1.0/pb.dot(pb);
                let kx = kk * pa.dot(pb);
                let ky = kk * (2.0*pa.dot(pa)+pd.dot(pb)) / 3.0;
                let kz = kk * pd.dot(pa);

                let res;
                
                let p  = ky - kx*kx;
                let q  = kx * (2.0*kx*kx - 3.0*ky) + kz;
                let p3 = p*p*p;
                let q2 = q*q;
                let h  = q2 + (4.0*p3);

                if h >= 0.0 {
                    let h = h.sqrt();
                    let x = (vec2(h, -h) - q) / 2.0;
                    let uv = x.sign() * x.abs().powf(vec2(1.0/3.0, 1.0/3.0));
                    let t = (uv[0]+uv[1]-kx).clamp(0.0, 1.0);
                    let q = pd + (pc+pb*t)*t;
                    res = q.dot(q);
                } else {
                    let z = (-p).sqrt();
                    let v = (q / (p*z*2.0)).acos() / 3.0;
                    let m = v.cos();
                    let n = v.sin() * 1.732050808;
                    let t = (vec3(m+m, -n-m, n-m)*z-kx).clamp(0.0, 1.0);
                    let qx = pd + (pc+pb*t[0]) * t[0];
                    let dx = qx.dot(qx);
                    let qy = pd + (pc+pb*t[1]) * t[1];
                    let dy = qy.dot(qy);
                    res = dx.min(dy);
                }
                
                res.sqrt().abs()
            },
            Self::Curve { start, end, first_control, second_control } => {
                const STEPS: usize = 30;
                let solve_distance = |i, t, min_distance: &mut f32, closest_step: &mut usize| {
                    let curve_pt = compute_curve(t, start, end, first_control, second_control);

                    let x = p[0]-curve_pt[0];
                    let y = p[1]-curve_pt[1];
                    let distance = x*x + y*y;   // No need to square the distance everytime, we do it once at the end

                    if distance < *min_distance {
                        *min_distance = distance;
                        *closest_step = i;
                    }
                };
                
                // Brute force method, because a closed-form solution would be too complex
                // see for yourself: https://www.shadertoy.com/view/4sKyzW
                let mut min_distance = f32::MAX;
                let mut closest_step = 0;
                
                // Step 1: Coarse check
                let coarse_step_value = 1.0 / STEPS as f32;
                for i in 0..=STEPS {
                    let t = coarse_step_value * (i as f32);
                    solve_distance(i, t, &mut min_distance, &mut closest_step);
                }

                // Step 2: fine check
                let bounds_min = match closest_step == 0 {
                    true => 0.0,
                    false => (closest_step - 1) as f32 * coarse_step_value,
                };

                let bounds_max = match closest_step == STEPS {
                    true => 1.0,
                    false => (closest_step + 1) as f32 * coarse_step_value,
                };

                let fine_step = (bounds_max - bounds_min) / STEPS as f32;
                for i in 0..=STEPS {
                    let t = bounds_min + (i as f32 * fine_step);
                    solve_distance(i, t, &mut min_distance, &mut closest_step)
                }

                min_distance.sqrt().abs()
            }
        }
    }

    /// Write up to 3 intersections in `out` at height `y`
    pub fn intersections(&self, y: f32, out: &mut [f32; 3]) -> usize {
        match *self {
            Self::Line { start, end } => {
                if (y >= start[1] && y <= end[1]) || (y >= end[1] && y < start[1]) {
                    let h = (y-start[1])/(end[1]-start[1]);
                    out[0] = mix(start[0], end[0], h);
                    1
                } else {
                    0
                }
            },
            Self::Quad { mut start, mut end, mut control } => {
                // Implementation from https://github.com/Pomax/bezierjs
                let min_y = start[1].min(control[1]).min(end[1]);
                let max_y = start[1].max(control[1]).max(end[1]);
                if y < min_y || y > max_y {
                    return 0;
                }

                let x0 = start[0];
                let x1 = control[0];
                let x2 = end[0];
                let solve = |t: f32| {
                    let t2 = t * t;
                    let mt = 1.0-t;
                    let mt2 = mt * mt;
                    (x0 * mt2) + (x1 * 2.0*mt*t) + (x2 * t2)
                };

                align_quadratic(y, &mut start, &mut end, &mut control);

                let mut count = 0;
                let a = start[1];
                let b = control[1];
                let c = end[1];
                let d = a - 2.0 * b + c;

                if d != 0.0 {
                    let m1 = -(b*b - a*c).sqrt();
                    let m2 = -a + b;
                    let r0 = -(m1 + m2) / d;
                    let r1 = -(-m1 + m2) / d;

                    if 0.0 <= r0 && r0 <= 1.0 {
                        out[count] = solve(r0);
                        count += 1;
                    }

                    if r0 != r1 && 0.0 <= r1 && r1 <= 1.0 {
                        out[count] = solve(r1);
                        count += 1;
                    }
                } else if b != c && d == 0.0 {
                    let r0 = (2.0 * b - c) / (2.0 * b - 2.0 * c);
                    if 0.0 <= r0 && r0 <= 1.0 {
                        count = 1;
                        out[0] = solve(r0);
                    }
                }

                return count;
            },
            Self::Curve { mut start, mut end, mut first_control, mut second_control } => {
                // Implementation from https://github.com/Pomax/bezierjs
                let crt = |v: f32| {
                    if v < 0.0 {
                        -((-v).powf(1.0/3.0))
                    } else {
                        v.powf(1.0/3.0)
                    }
                };

                let x0 = start[0];
                let x1 = first_control[0];
                let x2 = second_control[0];
                let x3 = end[0];
                let solve = |t: f32| {
                    let t2 = t * t;
                    let t3 = t2 * t;
                    let mt = 1.0-t;
                    let mt2 = mt * mt;
                    let mt3 = mt2 * mt;
                    (x0*mt3) + (3.0*x1*mt2*t) + (3.0*x2*mt*t2) + (x3*t3)
                };
                
                align_cubic(y, &mut start, &mut end, &mut first_control, &mut second_control);
                
                let mut count = 0;

                let pa = start[1];
                let pb = first_control[1];
                let pc = second_control[1];
                let pd = end[1];

                let d = -pa + 3.0 * pb - 3.0 * pc + pd;
                let mut a = 3.0 * pa - 6.0 * pb + 3.0 * pc;
                let mut b = -3.0 * pa + 3.0 * pb;
                let mut c = pa;

                a /= d;
                b /= d;
                c /= d;

                let p = (3.0 * b - a * a) / 3.0;
                let p3 = p / 3.0;
                let q = (2.0 * a * a * a - 9.0 * a * b + 27.0 * c) / 27.0;
                let q2 = q / 2.0;
                let discriminant = q2 * q2 + p3 * p3 * p3;

                if discriminant < 0.0 {
                    let tau = 2.0 * ::std::f32::consts::PI;
                    let mp3 = -p / 3.0;
                    let mp33 = mp3 * mp3 * mp3;
                    let r = mp33.sqrt();
                    let t = -q / (r * 2.0);
                    let cosphi = t.clamp(-1.0, 1.0);
                    let phi = cosphi.acos();
                    let crtr = crt(r);
                    let t1 = 2.0 * crtr;
                    
                    let r0 = t1 * (phi / 3.0).cos() - a / 3.0;
                    let r1 = t1 * ((phi + tau) / 3.0).cos() - a / 3.0;
                    let r2 = t1 * ((phi + 2.0 * tau) / 3.0).cos() - a / 3.0;

                    if 0.0 <= r0 && r0 <= 1.0 {
                        out[count] = solve(r0);
                        count += 1;
                    }

                    if 0.0 <= r1 && r1 <= 1.0 {
                        out[count] = solve(r1);
                        count += 1;
                    }

                    if 0.0 <= r2 && r2 <= 1.0 {
                        out[count] = solve(r2);
                        count += 1;
                    }

                } else if discriminant == 0.0 {
                    let u1 = match q2 < 0.0 {
                        true => crt(-q2),
                        false => -crt(q2)
                    };

                    let r0 = 2.0 * u1 - a / 3.0;
                    let r1 = -u1 - a / 3.0;
                    if 0.0 <= r0 && r0 <= 1.0 {
                        out[count] = solve(r0);
                        count += 1;
                    }

                    if r0 != r1 && 0.0 <= r1 && r1 <= 1.0 {
                        out[count] = solve(r1);
                        count += 1;
                    }
                } else {
                    let sd = discriminant.sqrt();
                    let u1 = crt(-q2 + sd);
                    let v1 = crt(q2 + sd);
                    let r = u1 - v1 - a / 3.0;
                    if 0.0 <= r && r <= 1.0 {
                        out[count] = solve(r);
                        count += 1;
                    }
                }

                count
            }
        }
    }

    /// Normalize the curve in place using the values provided. Assumes the curves coordinates are in a `0..width` and `0..height` range
    /// See also [Line::normalize_to]
    pub fn normalize(&mut self, width: f32, height: f32) {
        *self = self.normalize_to(width, height);
    }

    /// Normalize the curve in place using the value provided. Assumes the curves coordinates are in `x..width` and `y..height`
    /// See also [Line::normalize_to_with_offset]
    pub fn normalize_with_offset(&mut self, x: f32, y: f32, width: f32, height: f32) {
        *self = self.normalize_to_with_offset(x, y, width, height);
    }

    /// Return the same line with the coordinates normalized using `width` and `height`
    pub fn normalize_to(&self, width: f32, height: f32) -> Self {
        let p = vec2(width, height);
        match *self {
            Self::Line { start, end } => Self::Line { start: start / p, end: end / p },
            Self::Quad { start, end, control } => Self::Quad { start: start / p, end: end / p, control: control / p },
            Self::Curve { start, end, first_control, second_control } => Self::Curve { start: start / p, end: end / p, first_control: first_control / p,  second_control: second_control / p }
        }
    }

    /// Return the same line with the coordinates normalized using `width` and `height`
    pub fn normalize_to_with_offset(&self, x: f32, y: f32, width: f32, height: f32) -> Self {
        let o = vec2(x, y);
        let p = vec2(width, height);
        match *self {
            Self::Line { start, end } => Self::Line { start: (start-o) / p, end: (end-o) / p },
            Self::Quad { start, end, control } => Self::Quad { start: (start-o) / p, end: (end-o) / p, control: (control-o) / p },
            Self::Curve { start, end, first_control, second_control } => Self::Curve { 
                start: (start-o) / p,
                end: (end-o) / p,
                first_control: (first_control - o) / p,
                second_control: (second_control - o) / p 
            }
        }
    }

    /// Flip the y component. Assumes the line has been normalized
    pub fn flip_y(&mut self) {
        let p1 = vec2(1.0, -1.0);
        let p2 = vec2(0.0, -1.0);
        *self = match *self {
            Self::Line { start, end } => Self::Line { start: (start * p1) - p2, end: (end * p1) - p2 },
            Self::Quad { start, end, control } => Self::Quad { start: (start * p1) - p2, end: (end * p1) - p2, control: (control * p1) - p2 },
            Self::Curve { start, end, first_control, second_control } => Self::Curve { 
                start: (start * p1) - p2,
                end: (end * p1) - p2,
                first_control: (first_control * p1) - p2,
                second_control: (second_control * p1) - p2
            }
        };
    }

}

fn compute_curve(t: f32, start: Vec2, end: Vec2, control1: Vec2, control2: Vec2) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0-t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;
    
    vec2(
        (start[0]*mt3) + (3.0*control1[0]*mt2*t) + (3.0*control2[0]*mt*t2) + (end[0]*t3),
        (start[1]*mt3) + (3.0*control1[1]*mt2*t) + (3.0*control2[1]*mt*t2) + (end[1]*t3),
    )
}

fn align_quadratic(y: f32, start: &mut Vec2, end: &mut Vec2, control: &mut Vec2) {
    let p = vec2(0.0, y);
    *start = *start - p;
    *end = *end - p;
    *control = *control - p;
}

fn align_cubic(y: f32, start: &mut Vec2, end: &mut Vec2, control_1: &mut Vec2, control_2: &mut Vec2) {
    let p = vec2(0.0, y);
    *start = *start - p;
    *end = *end - p;
    *control_1 = *control_1 - p;
    *control_2 = *control_2 - p;
}
