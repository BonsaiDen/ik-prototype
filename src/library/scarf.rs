// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use ::{Vec2, Space, ParticleSystem, ParticleTemplate, Skeleton};
use ::library::{Accessory, Renderer, Collider};


// A Scarf --------------------------------------------------------------------
pub struct Scarf {
    bone: &'static str,
    particles: ParticleSystem,
    color: u32,
    offset: Vec2,
    facing: Vec2,
    gravity: Vec2
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
        }
    }

}

impl<R: Renderer, C: Collider> Accessory<R, C> for Scarf {

    fn set_bone(&mut self, bone: &'static str) {
        self.bone = bone;
    }

    fn attach(&mut self, _: &Skeleton) {
        self.particles.visit_particles_mut(|_, p| {
            p.set_position(Vec2::zero());
        });
    }

    fn attached(&self) -> bool {
        true
    }

    fn detach(&mut self, _: &Skeleton) {}

    fn apply_force(&mut self, _: Vec2) {}

    fn get_iks(&self, _: &Skeleton) -> Option<Vec<(&'static str, Vec2, bool)>> {
        None
    }

    fn fixate(&mut self, skeleton: &Skeleton) {
        // TODO simpler way to get the offset?
        let origin = skeleton.bone_end(Space::Local, self.bone);
        let offset = skeleton.bone_end(Space::World, self.bone) - origin;
        self.facing = skeleton.local_transform();
        self.particles.get_mut(0).set_position(origin);
        self.offset = offset;
    }

    fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    fn step(&mut self, renderer: &R, collider: &C) {

        // Don't let the scarf fall into rest
        self.particles.activate();

        let world_offset = self.offset;
        self.particles.step(
            renderer.dt(),
            Vec2::new(-200.0 * self.facing.x, (renderer.time() * 4.0).sin() * self.gravity.y * 0.5),
            |p| {
                if let Some((pos, _, _)) = collider.world(p.position + world_offset) {
                    p.position = pos - world_offset;
                }
            }
        );

    }

    fn draw(&self, renderer: &mut R) {
        self.particles.visit_particles_chained(|_, p, n| {
            renderer.draw_line(self.offset + p.position, self.offset + n.position, self.color);
        });
    }

}

