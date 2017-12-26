// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// External Dependencies ------------------------------------------------------
use downcast_rs::Downcast;


// Internal Dependencies ------------------------------------------------------
use ::{Skeleton, Vec2};


// Modules --------------------------------------------------------------------
mod scarf;
pub use self::scarf::Scarf;

mod weapon;
pub use self::weapon::Weapon;

mod stick_figure;
pub use self::stick_figure::{
    StickFigureConfig, StickFigureState, StickFigure
};


// Traits ---------------------------------------------------------------------
pub trait Renderer {
    fn dt(&self) -> f32;
    fn draw_line(&mut self, start: Vec2, end: Vec2, color: u32);
    fn draw_circle(&mut self, c: Vec2, r: f32, color: u32);
    fn draw_rect(&mut self, tr: Vec2, bl: Vec2, color: u32);
}

pub trait Collider {
    fn world(&self, &mut Vec2) -> bool;
    fn local(&self, &mut Vec2) -> bool;
}

pub trait Accessory<R: Renderer, C: Collider>: Downcast {
    fn set_bone(&mut self, &'static str);
    fn attach(&mut self, skeleton: &Skeleton);
    fn attached(&self) -> bool;
    fn detach(&mut self, skeleton: &Skeleton);
    fn apply_force(&mut self, force: Vec2);
    fn get_iks(&self, skeleton: &Skeleton) -> Option<Vec<(&'static str, Vec2, bool)>>;
    fn fixate(&mut self, skeleton: &Skeleton);
    fn set_gravity(&mut self, gravity: Vec2);
    fn step(&mut self, f32, &C);
    fn draw(&self, renderer: &mut R);
}

impl_downcast!(Accessory<R, C> where R: Renderer, C: Collider);

