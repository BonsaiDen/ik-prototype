// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use lean::{Vec2, ParticleSystem, ParticleTemplate};
use lean::library::{Attachement, LeanRenderer, LineRenderer, CircleRenderer};


// A Scarf --------------------------------------------------------------------
pub struct Scarf {
    particles: ParticleSystem,
    offset: Vec2,
    gravity: Vec2,
    timer: f32
}

impl Scarf {

    pub fn new(length: f32, segments: usize) -> Self {
        Self {
            particles: ParticleTemplate::schal(
                1,
                segments,
                length / segments as f32,
                Vec2::zero()
            ),
            offset: Vec2::zero(),
            gravity: Vec2::zero(),
            timer: 0.0
        }
    }

}

impl Attachement for Scarf {

    fn attach_with_offset(&mut self, origin: Vec2, offset: Vec2) {
        self.particles.get_mut(0).set_position(origin);
        self.offset = offset;
    }

    fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    fn step<
        C: Fn(&mut Vec2) -> bool,
        D: Fn(&mut Vec2) -> bool

    >(&mut self, dt: f32, collider_local: &C, _: &D) {

        self.timer += dt;

        // Don't let the scarf fall into rest
        self.particles.activate();

        self.particles.step(
            dt,
            Vec2::new(self.gravity.x, (self.timer * 4.0).sin() * self.gravity.y),
            |p| {
                collider_local(&mut p.position);
            }
        );

    }

    fn draw<R: LeanRenderer + LineRenderer + CircleRenderer>(&self, renderer: &mut R) {
        self.particles.visit_particles_chained(|i, p, n| {
            renderer.draw_line(self.offset + p.position, self.offset + n.position, 0x00ff_ff00);
        });
    }

    fn reset(&mut self) {
        self.particles.visit_particles_mut(|_, p| {
            p.set_position(Vec2::zero());
        });
    }

}

