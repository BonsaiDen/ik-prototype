// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::collections::{HashMap, HashSet};
use super::{Constraint, StickConstraint, Particle, ParticleSystem, Vec2};


// Skeleton Ragdoll Abstraction -----------------------------------------------
pub struct Ragdoll {
    joints: Vec<Particle>,
    constraints: Vec<Box<Constraint>>,
    constraint_names: Vec<String>,
    constraint_name_map: HashMap<String, usize>,
    joint_constraint_map: HashMap<usize, Vec<usize>>,
    steps_until_rest: usize,
    bounds: (Vec2, Vec2)
}

impl Ragdoll {

    pub fn new(joints: Vec<Particle>, constraints: Vec<Box<Constraint>>) -> Self {

        let mut ragdoll = Self {
            joints,
            constraints,
            constraint_names: Vec::new(),
            constraint_name_map: HashMap::new(),
            joint_constraint_map: HashMap::new(),
            steps_until_rest: 10,
            bounds: (Vec2::zero(), Vec2::zero())
        };

        ragdoll.rebuild_constraints();
        ragdoll

    }


    // Getters ----------------------------------------------------------------
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

    // Others -----------------------------------------------------------------
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

    // Forces -----------------------------------------------------------------
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

    pub fn split_bone_from_parent(&mut self, name: &str) {
        self.split_off_joint(name, None);
    }

    // Internal ---------------------------------------------------------------
    fn split_off_joint(&mut self, name: &str, at_length: Option<f32>) {

        let ci = *&self.constraint_name_map[name];
        let (end, start) = {
            let constraint = &self.constraints[ci];
            (
                constraint.first_particle(),
                constraint.second_particle()
            )
        };

        // Separate the ragdoll into two sets of joints
        // One to the left of the constraint to be split...
        let mut left_points = HashSet::new();
        self.find_points_behind_constraint(ci, end, &mut left_points);

        // And one to the right of the constraint to be split.
        let mut right_points = HashSet::new();
        self.find_points_behind_constraint(ci, start, &mut right_points);

        // Now we remove all non-visual constraints which were linking between those two sets
        self.constraints.retain(|c| {
            let (l, r) = (c.first_particle(), c.second_particle());
            let is_crossing = !c.visual() && (left_points.contains(&l) && right_points.contains(&r))
                           || (left_points.contains(&r) && right_points.contains(&l));

            // Retain only non set crossing constraints
            !is_crossing
        });

        // Split constraint at length...
        if let Some(_) = at_length {

            // TODO


        // ...or remove joint from parent socket...
        } else {

            // We duplicate the joint and insert it into our list of points
            let new_joint = self.joints[start].clone();
            let new_index = self.joints.len();
            self.joints.push(new_joint);

            // Then create a new constraint to replace the existing one
            let rest_length = (self.joints[new_index].position - self.joints[end].position).len();
            let mut c = StickConstraint::new(
                self.constraints[ci].name().to_string(),
                end,
                new_index,
                rest_length
            );
            c.set_visual(true);

            // Finally replace old constraint with the new one that using the duplicated point
            self.constraints[ci] = Box::new(c);

        }

        // Rebuild joint constraint map
        self.rebuild_constraints();

    }

    fn rebuild_constraints(&mut self) {

        self.joint_constraint_map.clear();
        self.constraint_name_map.clear();
        self.constraint_names.clear();

        for index in 0..self.joints.len() {
            self.joint_constraint_map.insert(index, Vec::new());
        }

        for (index, c) in self.constraints.iter().enumerate() {
            self.joint_constraint_map.get_mut(&c.first_particle()).unwrap().push(index);
            self.joint_constraint_map.get_mut(&c.second_particle()).unwrap().push(index);
            self.constraint_name_map.insert(c.name().to_string(), index);
            self.constraint_names.push(c.name().to_string());
        }

    }

    fn find_points_behind_constraint(&self, constraint: usize, joint: usize, points: &mut HashSet<usize>) {

        // Get all constraints connected to the current joint
        let constraints = &self.joint_constraint_map[&joint];

        // Add current joint to list
        points.insert(joint);

        // Search through all further constraints
        for ci in constraints {
            if *ci != constraint {

                let constraint = &self.constraints[*ci];
                if constraint.visual() {

                    let end = constraint.first_particle();
                    let start = constraint.second_particle();

                    if !points.contains(&end) {
                        self.find_points_behind_constraint(*ci, end, points);
                    }

                    if !points.contains(&start) {
                        self.find_points_behind_constraint(*ci, start, points);
                    }

                }

            }
        }

    }

}

