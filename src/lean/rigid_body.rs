// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// Internal Dependencies ------------------------------------------------------
use super::{StickConstraint, Particle, ParticleSystem, Vec2};


// Particle based Rigid Bodies ------------------------------------------------
pub struct RigidBodyData {
    pub points: Vec<(&'static str, f32, f32)>,
    pub constraints: Vec<(&'static str, &'static str, bool)>
}

pub struct RigidBody {
    angle: f32,
    position: Vec2,
    offset: Vec2,
    scale: Vec2,
    lines: Vec<((Vec2, usize), (Vec2, usize), bool)>,
    particles: ParticleSystem
}

impl RigidBody {

    pub fn new(data: &'static RigidBodyData) -> Self {

        let mut particles = ParticleSystem::new(data.points.len(), 4);
        let mut points = HashMap::new();
        for (index, p) in data.points.iter().enumerate() {
            points.insert(p.0, (Vec2::new(p.1, p.2), index));
        }

        let mut lines = Vec::new();
        for c in &data.constraints {

            let a = points[c.0];
            let b = points[c.1];
            let l = (a.0 - b.0).len();

            let mut constraint = StickConstraint::new(a.1, b.1, l);
            if c.2 {
                constraint.set_visible(true);
            }
            lines.push((a, b, c.2));
            particles.add_constraint(constraint);

        }

        Self {
            angle: 0.0,
            position: Vec2::zero(),
            offset: Vec2::zero(),
            scale: Vec2::new(1.0, 1.0),
            lines: lines,
            particles: particles
        }
    }

    // Static (Data Based) ----------------------------------------------------
    pub fn step_static(&mut self, p: Vec2, offset: Vec2, scale: Vec2, angle: f32) {
        self.position = p;
        self.offset = offset;
        self.scale = scale;
        self.angle = angle;
    }

    pub fn visit_static<C: FnMut(Vec2, Vec2)>(&self, mut callback: C) {
        for &(a, b, visual) in &self.lines {
            if visual {
                callback(
                    (a.0 + self.offset).scale(self.scale).rotate(self.angle) + self.position,
                    (b.0 + self.offset).scale(self.scale).rotate(self.angle) + self.position
                );
            }
        }
    }

    // Dynamic (Particle Based) -----------------------------------------------
    pub fn make_dynamic(&mut self) {
        self.particles.activate();
        for &(a, b, _) in &self.lines {
            let pa = (a.0 + self.offset).scale(self.scale).rotate(self.angle) + self.position;
            let pb = (b.0 + self.offset).scale(self.scale).rotate(self.angle) + self.position;
            self.particles.get_mut(a.1).set_position(pa);
            self.particles.get_mut(b.1).set_position(pb);
            self.particles.get_mut(a.1).set_invmass(1.0);
            self.particles.get_mut(b.1).set_invmass(1.0);
        }
    }

    pub fn apply_dynamic_force(&mut self, force: Vec2) {
        self.particles.get_mut(0).apply_force(force);
    }

    pub fn step_dynamic<C: Fn(&mut Particle)>(&mut self, time_step: f32, gravity: Vec2, collision: C) {
        self.particles.step(time_step, gravity, collision);
    }

    pub fn visit_dynamic<C: FnMut((usize, Vec2), (usize, Vec2), bool)>(&self, callback: C) {
        self.particles.visit_constraints(callback);
    }

}

