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
    bone: &'static str,
    particles: ParticleSystem,
    color: u32,
    offset: Vec2,
    facing: Vec2,
    gravity: Vec2,
    timer: f32
}

impl Scarf {

    pub fn new(length: f32, segments: usize, color: u32) -> Self {
        Self {
            bone: "Root",
            particles: ParticleTemplate::schal(
                1,
                segments,
                length / segments as f32,
                Vec2::zero()
            ),
            color: color,
            offset: Vec2::zero(),
            gravity: Vec2::zero(),
            facing: Vec2::new(1.0, 1.0),
            timer: 0.0
        }
    }

}

impl<R: Renderer, C: Collider> Attachement<R, C> for Scarf {

    fn set_bone(&mut self, bone: &'static str) {
        self.bone = bone;
    }

    fn loosen(&mut self, _: &Skeleton) {}

    fn fasten(&mut self, _: &Skeleton) {
        self.particles.visit_particles_mut(|_, p| {
            p.set_position(Vec2::zero());
        });
    }

    fn apply_force(&mut self, _: Vec2) {}

    // TODO Figure out how to cleanly allow access to custom figure properties
    fn get_iks(&self, _: &Skeleton, _: f32, _: f32) -> Option<Vec<(&'static str, Vec2, bool)>> {
        None
    }

    fn fixate(&mut self, skeleton: &Skeleton, _: f32, _: f32) {
        // TODO set attachment bone from the outside
        let origin = skeleton.get_bone_end_local(self.bone);
        let offset = skeleton.get_bone_end_world(self.bone) - origin;
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
            renderer.draw_line(self.offset + p.position, self.offset + n.position, self.color);
        });
    }

}

