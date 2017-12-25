// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use ::lean::{Skeleton, Vec2};


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
pub trait Renderer {
    fn dt(&self) -> f32;
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: u32);
    fn draw_circle(&mut self, c: Vec2, r: f32, color: u32);
}

pub trait Collider {
    fn world(&self, &mut Vec2) -> bool;
    fn local(&self, &mut Vec2) -> bool;
}

pub trait Attachement<R: Renderer, C: Collider> {
    fn loosen(&mut self, skeleton: &Skeleton);
    fn apply_force(&mut self, force: Vec2);
    fn fixate(&mut self, skeleton: &Skeleton);
    fn set_gravity(&mut self, gravity: Vec2);
    fn step(&mut self, f32, &C);
    fn draw(&self, renderer: &mut R);
    fn reset(&mut self);
}

