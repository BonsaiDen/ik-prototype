// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;
use std::ops::{Add, Sub, Mul, Div};


// 2D Vector Abstraction ------------------------------------------------------
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32
}

impl Vec2 {

    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y
        }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0)
    }

    pub fn unit(&self) -> Vec2 {
        *self / self.len()
    }

    pub fn angle(&self) -> f32 {
        self.y.atan2(self.x)
    }

    pub fn angle_between(&self, other: Vec2) -> f32 {
        (*self * other / (self.len() * other.len())).acos()
    }

    pub fn scale(&self, unit: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * unit.x,
            y: self.y * unit.y
        }
    }

    pub fn cross(&self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x * other.y - other.x * self.y,
            y: self.y * other.x - other.y * self.x
        }
    }

    pub fn len(&self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn flipped(&self) -> Vec2 {
        Vec2 {
            x: self.y,
            y: self.x
        }
    }

    pub fn len_squar(&self) -> f32 {
        self.x + self.y
    }

    pub fn rotate(&self, a: f32) -> Vec2 {
        let c = a.cos();
        let s = a.sin();
        Vec2 {
            x: self.x * c - self.y * s,
            y: self.x * s + self.y * c
        }
    }

}

impl Add for Vec2 {

    type Output = Vec2;

    fn add(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x + other.x,
            y: self.y + other.y
        }
    }

}

impl Sub for Vec2 {

    type Output = Vec2;

    fn sub(self, other: Vec2) -> Vec2 {
        Vec2 {
            x: self.x - other.x,
            y: self.y - other.y
        }
    }

}

impl Mul for Vec2 {

    type Output = f32;

    fn mul(self, other: Vec2) -> f32 {
        self.x * other.x + self.y * other.y
    }

}

impl Mul<f32> for Vec2 {

    type Output = Vec2;

    fn mul(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x * scalar,
            y: self.y * scalar
        }
    }

}

impl Div<f32> for Vec2 {

    type Output = Vec2;

    fn div(self, scalar: f32) -> Vec2 {
        Vec2 {
            x: self.x / scalar,
            y: self.y / scalar
        }
    }

}


// Angle Abstraction ----------------------------------------------------------
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Facing {
    Right,
    Left
}

impl Facing {

    pub fn to_vec(&self) -> Vec2 {
        match *self {
            Facing::Right => Vec2::new(1.0, 1.0),
            Facing::Left => Vec2::new(-1.0, 1.0)
        }
    }

}

#[derive(Debug, Clone, Copy)]
pub struct Angle {
    r: f32
}

impl Angle {

    pub fn transform(r: f32, v: Vec2) -> f32 {
        if v.x == -1.0 {
            -r - PI

        } else {
            r
        }
    }

    pub fn facing(mut r: f32) -> Facing {
        if r > PI {
            r -= PI * 2.0;
        }
        if r >= 0.0 {
            Facing::Right

        } else {
            Facing::Left
        }
    }

    pub fn interpolate(mut from: f32, to: f32, limit: f32) -> f32 {

        let r = to - from;
        let dr = r.sin().atan2(r.cos());

        from += dr.min(limit).max(-limit);

        if from > PI * 2.0 {
            from -= PI * 2.0;

        } else if from < 0.0 {
            from += PI * 2.0;
        }

        from

    }

    pub fn offset(r: f32, distance: f32) -> Vec2 {
        Angle::from_radians(r).to_vec(distance)
    }

    fn from_radians(r: f32) -> Self {
        Self {
            r: r
        }
    }

    fn to_unit_vec(&self) -> Vec2 {
        Vec2::new(self.r.cos(), self.r.sin())
    }

    fn to_vec(&self, distance: f32) -> Vec2 {
        self.to_unit_vec() * distance
    }

}

