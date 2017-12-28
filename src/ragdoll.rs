// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;
use super::{Constraint, Particle, ParticleSystem, Vec2};


// Skeleton Ragdoll Abstraction -----------------------------------------------
pub struct RagdollConstraint {

}

pub struct Ragdoll {
    joints: Vec<Particle>,
    constraints: Vec<Box<Constraint>>,
    constraint_names: Vec<String>,
    joint_contraint_map: HashMap<usize, Vec<usize>>,
    steps_until_rest: usize,
    bounds: (Vec2, Vec2)
}

impl Ragdoll {

    pub fn new(joints: Vec<Particle>, named_constraints: Vec<(String, Box<Constraint>)>) -> Self {

        let mut constraints = Vec::new();
        let mut constraint_names = Vec::new();
        let mut joint_contraint_map = HashMap::new();

        // Bone joints
        for (index, _) in joints.iter().enumerate() {
            joint_contraint_map.insert(index, Vec::new());
        }

        // Bone constraints
        for (index, (name, c)) in named_constraints.into_iter().enumerate() {
            joint_contraint_map.get_mut(&c.first_particle()).unwrap().push(index);
            joint_contraint_map.get_mut(&c.second_particle()).unwrap().push(index);
            constraints.push(c);
            constraint_names.push(name);
        }

        Self {
            joints,
            constraints,
            constraint_names,
            joint_contraint_map,
            steps_until_rest: 10,
            bounds: (Vec2::zero(), Vec2::zero())
        }

    }

    pub fn step<C: Fn(&mut Particle)>(&mut self, dt: f32, gravity: Vec2, collider: C) {

        if self.steps_until_rest == 0 {
            return;
        }

        ParticleSystem::accumulate_forces(gravity, &mut self.joints[..]);
        ParticleSystem::verlet(dt, &mut self.joints[..]);

        if !ParticleSystem::satisfy_constraints(
            1,
            &mut self.joints[..],
            &self.constraints[..],
            &mut self.bounds,
            collider
        ) {
            self.steps_until_rest = self.steps_until_rest.saturating_sub(1);
        }

    }

    pub fn visit<C: FnMut(Vec2, Vec2, &str)>(&mut self, mut callback: C) {
        for (index, c) in self.constraints.iter().enumerate() {
            if c.visual() {
                let name = &self.constraint_names[index];
                let start = self.joints[c.first_particle()].position;
                let end = self.joints[c.first_particle()].position;
                callback(start, end, name);
            }
        }
    }

    // TODO visit constraints instead

    /*
    pub fn split_radgoll(&mut self, bone: &'static str) {
        if let Some(bone) = self.get_bone(bone) {
            if let Some(parent) = bone.parent() {

                // Get current start and end particles
                let start = self.bones[parent].particle_index();
                let end = bone.particle_index();

                // Find the bone's constraint
                let ci = self.constraints.iter().position(|c| {
                     c.typ() == ConstraintType::Stick && c.first_particle() == start && c.second_particle() == end
                });

                if let Some(ci) = ci {

                    let constraint = &self.constraints[ci];

                    // Clone start point

                    // Update bone and constraint start point

                }

            }
        }
    }*/

}

