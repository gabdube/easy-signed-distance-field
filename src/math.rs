/// Simple vec library to not bloat the project with a huge math dependency

use std::ops::*;

#[repr(transparent)]
#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub struct Vec2 { v: [f32; 2] }
pub type Point = Vec2;

#[inline]
pub fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2 { v: [x, y] }
}

impl Vec2 {
    #[inline(always)]
    pub fn dot(&self, other: Self) -> f32 {
        self[0] * other[0] + self[1] * other[1]
    }

    #[inline(always)]
    pub fn cross(&self, other: Self) -> f32 {
        self[0]*other[1]-self[1]*other[0]
    }

    #[inline(always)]
    pub fn length(&self) -> f32 {
        let x = self[0];
        let y = self[1];
        ((x*x)+(y*y)).sqrt()
    }

    #[inline(always)]
    pub fn sign(&self) -> Self {
        let mut x = self[0];
        x = if x < 0.0 {
            -1.0
        } else if x == 0.0 {
            0.0
        } else {
            1.0
        };

        let mut y = self[1];
        y = if y < 0.0 {
            -1.0
        } else if y == 0.0 {
            0.0
        } else {
            1.0
        };

        vec2(x, y)
    }

    #[inline(always)]
    pub fn normalize(&self) -> Self {
        let len = self.length();
        if len == 0.0 {
            vec2(0.0, 1.0);
        }
        return vec2(self[0]/len, self[1]/len);
    }

    #[inline(always)]
    pub fn get_orthonormal(&self, polarity: bool) -> Self {
        let x = self[0];
        let y = self[1];
        let len = self.length();

        if len == 0.0 {
            match polarity {
                true => vec2(0.0, 1.0),
                false => vec2(0.0, -1.0)
            }
        } else {
            match polarity {
                true => vec2(-y/len, x/len),
                false => vec2(y/len, -x/len)
            }
        }
    }

    #[inline(always)]
    pub fn abs(&self) -> Self {
        vec2(self[0].abs(), self[1].abs())
    }

    #[inline(always)]
    pub fn powf(&self, other: Self) -> Self {
        vec2(self[0].powf(other[0]), self[1].powf(other[1]))
    }

}

impl Add for Vec2 {
    type Output = Self;
    fn add(self, other: Self) -> Self::Output {
        vec2(self[0] + other[0], self[1] + other[1])
    }
}

impl Sub for Vec2 {
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output {
        vec2(self[0] - other[0], self[1] - other[1])
    }
}

impl Sub<f32> for Vec2 {
    type Output = Self;
    fn sub(self, other: f32) -> Self::Output {
        vec2(self[0] - other, self[1] - other)
    }
}


impl Mul for Vec2 {
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output {
        vec2(self[0] * other[0], self[1] * other[1])
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        vec2(self[0] * other, self[1] * other)
    }
}

impl Div for Vec2 {
    type Output = Self;
    fn div(self, other: Self) -> Self::Output {
        vec2(self[0] / other[0], self[1] / other[1])
    }
}

impl Div<f32> for Vec2 {
    type Output = Self;
    fn div(self, other: f32) -> Self::Output {
        vec2(self[0] / other, self[1] / other)
    }
}

impl Index<usize> for Vec2 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 1  => &self.v[index],
            _ => panic!("Vec2 index value must be 0 or 1")
        }
    }
}

impl IndexMut<usize> for Vec2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 | 1 => &mut self.v[index],
            _ => panic!("Vec2 index value must be 0 or 1")
        }
    }
}


/// Only used internally and only has the bare minimum feature, so it's not exposed
#[repr(transparent)]
#[derive(Default, Copy, Clone, Debug, PartialEq)]
pub(crate) struct Vec3 { v: [f32; 3] }

impl Vec3 {

    pub fn clamp(&self, min: f32, max: f32) -> Self {
        vec3(self[0].clamp(min, max), self[1].clamp(min, max), self[2].clamp(min, max))
    }

}


pub(crate) fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3 { v: [x, y, z] }
}

impl Sub<f32> for Vec3 {
    type Output = Self;
    fn sub(self, other: f32) -> Self::Output {
        vec3(self[0] - other, self[1] - other, self[2] - other)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;
    fn mul(self, other: f32) -> Self::Output {
        vec3(self[0] * other, self[1] * other, self[2] * other)
    }
}

impl Index<usize> for Vec3 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 | 1 | 2 => &self.v[index],
            _ => panic!("Vec2 index value must be 0, 1, or 2")
        }
    }
}
