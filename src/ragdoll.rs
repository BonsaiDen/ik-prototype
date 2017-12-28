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
pub struct Ragdoll {
    joints: Vec<Particle>,
    constraints: Vec<Box<Constraint>>,
    constraint_names: Vec<String>,
    constraint_name_map: HashMap<String, usize>,
    joint_contraint_map: HashMap<usize, Vec<usize>>,
    steps_until_rest: usize,
    bounds: (Vec2, Vec2)
}

impl Ragdoll {

    pub fn new(joints: Vec<Particle>, named_constraints: Vec<(String, Box<Constraint>)>) -> Self {

        let mut constraints = Vec::new();
        let mut constraint_names = Vec::new();
        let mut constraint_name_map = HashMap::new();
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
            constraint_name_map.insert(name.clone(), index);
            constraint_names.push(name);
        }

        Self {
            joints,
            constraints,
            constraint_names,
            constraint_name_map,
            joint_contraint_map,
            steps_until_rest: 10,
            bounds: (Vec2::zero(), Vec2::zero())
        }

    }

    pub fn at_rest(&self) -> bool {
        self.steps_until_rest == 0
    }

    pub fn bounds(&self) -> (Vec2, Vec2) {
        self.bounds
    }

    pub fn constraint_points(&self, name: &str) -> (Vec2, Vec2) {
        if let Some(index) = self.constraint_name_map.get(name) {
            let c = &self.constraints[*index];
            (
                self.joints[c.first_particle()].position,
                self.joints[c.second_particle()].position
            )

        } else {
            (Vec2::zero(), Vec2::zero())
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

    pub fn visit<C: FnMut(Vec2, Vec2, &str)>(&self, mut callback: C) {
        for (index, c) in self.constraints.iter().enumerate() {
            if c.visual() {
                let name = &self.constraint_names[index];
                let start = self.joints[c.first_particle()].position;
                let end = self.joints[c.second_particle()].position;
                callback(start, end, name);
            }
        }
    }

    pub fn apply_force(&mut self, local_origin: Vec2, force: Vec2, width: f32) {

        // Strength
        let strength = force.len();
        if strength > 0.0 {

            // Direction of the force
            let dir = force.unit();

            // Calculate force for each joint
            for joint in &mut self.joints {

                // Distance from joint to origin
                let d = 1.0 / ((joint.position - local_origin).len() / width.max(1.0)).max(1.0);

                // Force applied to this joint
                joint.apply_force(dir * strength * d);

            }

        }

    }

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

