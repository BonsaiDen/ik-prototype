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


// Types ----------------------------------------------------------------------
type RigidLine = (Vec2, usize);
type RigidPoint = (&'static str, f32, f32);
type RigidConstraint = (&'static str, &'static str, bool);
type RigidIK = (&'static str, f32, f32, bool);

// Particle based Rigid Bodies ------------------------------------------------
pub struct RigidBodyData {
    pub points: Vec<RigidPoint>,
    pub constraints: Vec<RigidConstraint>,
    pub iks: Vec<RigidIK>
}

pub struct RigidBody {
    angle: f32,
    position: Vec2,
    offset: Vec2,
    scale: Vec2,
    lines: Vec<(RigidLine, RigidLine, bool)>,
    iks: Vec<RigidIK>,
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
            let l = (a.0 - b.0).length();

            let mut constraint = StickConstraint::new(c.0.to_string(), a.1, b.1, l);
            if c.2 {
                constraint.set_visual(true);
            }
            lines.push((a, b, c.2));
            particles.add_constraint(constraint);

        }

        Self {
            angle: 0.0,
            position: Vec2::zero(),
            offset: Vec2::zero(),
            scale: Vec2::new(1.0, 1.0),
            iks: data.iks.clone(),
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

    pub fn iks_static(&self, offset: Vec2) -> Vec<(&'static str, Vec2, bool)> {
        self.iks.iter().map(|&(bone, x, y, positive)| {
            let p = Vec2::new(x, y);
            (
                bone,
                // Scale one for rotation, then scale back to work with skeletons
                // which always face to the right internally
                (p + self.offset).scale(self.scale).rotate(self.angle).scale(self.scale.flipped()) + offset,
                positive
            )

        }).collect()
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
        self.particles.activate();
        self.particles.get_mut(0).apply_force(force);
        self.particles.visit_particles_mut(|_, p| {
            p.set_invmass(1.0);
        });
    }

    pub fn step_dynamic<C: Fn(&mut Particle)>(&mut self, time_step: f32, gravity: Vec2, collision: C) {
        self.particles.step(time_step, gravity, collision);
    }

    pub fn visit_dynamic<C: FnMut((usize, Vec2), (usize, Vec2), bool)>(&self, callback: C) {
        self.particles.visit_constraints(callback);
    }

}

