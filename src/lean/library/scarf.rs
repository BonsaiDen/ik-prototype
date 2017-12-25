// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use lean::{Vec2, ParticleSystem, ParticleTemplate, Skeleton};
use lean::library::{Attachement, Renderer, Collider};


// A Scarf --------------------------------------------------------------------
pub struct Scarf {
    particles: ParticleSystem,
    offset: Vec2,
    facing: Vec2,
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
            facing: Vec2::new(1.0, 1.0),
            timer: 0.0
        }
    }

}

impl<R: Renderer, C: Collider> Attachement<R, C> for Scarf {

    fn loosen(&mut self, skeleton: &Skeleton) {}
    fn apply_force(&mut self, force: Vec2) {}

    fn fixate(&mut self, skeleton: &Skeleton) {
        // TODO set attachment bone from the outside
        let origin = skeleton.get_bone_end_local("Neck");
        let offset = skeleton.get_bone_end_world("Neck") - origin;
        self.facing = skeleton.local_transform();
        self.particles.get_mut(0).set_position(origin);
        self.offset = offset;
    }

    fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    fn step(&mut self, dt: f32, collider: &C) {

        self.timer += dt;

        // Don't let the scarf fall into rest
        self.particles.activate();
        self.particles.step(
            dt,
            Vec2::new(-200.0 * self.facing.x, (self.timer * 4.0).sin() * self.gravity.y * 0.5),
            |p| {
                collider.local(&mut p.position);
            }
        );

    }

    fn draw(&self, renderer: &mut R) {
        self.particles.visit_particles_chained(|_, p, n| {
            renderer.draw_line(self.offset + p.position, self.offset + n.position, 0x00ff_ff00);
        });
    }

    fn reset(&mut self) {
        self.particles.visit_particles_mut(|_, p| {
            p.set_position(Vec2::zero());
        });
    }

}

