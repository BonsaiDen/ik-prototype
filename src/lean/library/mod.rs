// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use ::lean::Vec2;


// Modules --------------------------------------------------------------------
mod scarf;
pub use self::scarf::Scarf;

mod standard_rifle;
pub use self::standard_rifle::StandardRifle;

mod stick_figure;
pub use self::stick_figure::{
    StickFigureConfig, StickFigureState, StickFigure
};


// Traits ---------------------------------------------------------------------
pub trait Attachement {
    fn attach_with_offset(&mut self, origin: Vec2, offset: Vec2);
    fn set_gravity(&mut self, gravity: Vec2);
    fn step<C: Fn(&mut Vec2) -> bool, D: Fn(&mut Vec2) -> bool>(&mut self, f32, &C, &D);
    fn draw<R: LeanRenderer + LineRenderer + CircleRenderer>(&self, renderer: &mut R);
    fn reset(&mut self);
}

pub trait LeanRenderer {
    fn dt(&self) -> f32;
}

pub trait LineRenderer {
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: u32);
}

pub trait CircleRenderer {
    fn draw_circle(&mut self, c: Vec2, r: f32, color: u32);
}

