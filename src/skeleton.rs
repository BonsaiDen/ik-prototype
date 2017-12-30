// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::EPSILON;
use std::collections::HashMap;


// Internal Dependencies ------------------------------------------------------
use super::{Angle, Space, Vec2, f32_equals};
use super::animation::{Animator, AnimatorBuilder, AnimationFrameBone};
use super::{
    Constraint, AngularConstraint, StickConstraint, Ragdoll, Particle
};


// Types ----------------------------------------------------------------------
pub enum SkeletalConstraint {
    Stick(&'static str, &'static str),
    Angular(&'static str, &'static str, &'static str, f32, f32),
}

type SkeletalBoneDescription = (
    // Parent, length, angle, ragdoll_inv_mass
    &'static str, f32, f32, f32, Option<f32>, Option<f32>
);
type SkeletalBone = (&'static str, SkeletalBoneDescription);
type RagdollBoneLink = (&'static str, &'static str);


// Skeleton Data Abstraction --------------------------------------------------
pub struct SkeletalData {
    pub bones: Vec<SkeletalBone>,
    pub ragdoll_parents: Vec<RagdollBoneLink>,
    pub constraints: Vec<SkeletalConstraint>
}

impl SkeletalData {

    fn to_internal_bones(&'static self) -> Vec<Bone> {

        // Generate initial bones
        let mut bones: Vec<Bone> = self.bones.iter().enumerate().map(|(index, bone)| {

            // Find parent bone index
            let mut parent = 255;
            for (i, p) in self.bones.iter().enumerate() {
                if p.0 == (bone.1).0
                    // Don't set the root bone as its own parent
                    && p.0 != bone.0 {
                    parent = i;
                    break;
                }
            }

            // Find ragdoll parent overrides
            let mut ragdoll_parent = parent;
            for &(name, parent) in &self.ragdoll_parents {
                if name == bone.0 {
                    for (i, p) in self.bones.iter().enumerate() {
                        if p.0 == parent {
                            ragdoll_parent = i;
                            break;
                        }
                    }
                }
            }

            Bone {
                index: index,
                parent: parent,
                ragdoll_parent: ragdoll_parent,
                children: Vec::new(),

                angle: 0.0,
                animation_angle: 0.0,
                offset_angle: 0.0,

                start: Vec2::zero(),
                end: Vec2::zero(),

                min_angle: (bone.1).4,
                max_angle: (bone.1).5,

                data: bone
            }

        }).collect();

        // Collect children
        let bone_children: Vec<Vec<usize>> = bones.iter().map(|bone| {
            let mut children = Vec::new();
            for other in &bones {
                if other.parent == bone.index
                    // Don't add the root bone as a child of itself
                    && other.index != other.parent {
                    children.push(other.index);
                }
            }
            children

        }).collect();

        // Assign children
        for (children, bone) in bone_children.into_iter().zip(bones.iter_mut()) {
            bone.children = children;
        }

        bones

    }

    fn to_animation_bones(&self) -> Vec<AnimationFrameBone> {
        self.bones.iter().map(|bone| {
            (bone.0, (bone.1).2)

        }).collect()
    }

    fn reset_animation_bones(&self, bones: &mut[AnimationFrameBone])  {
        for (bone, o) in self.bones.iter().zip(bones.iter_mut()) {
            o.1 = (bone.1).2;
        }
    }

}


// Skeleton Abstraction -------------------------------------------------------
pub struct Skeleton {

    // Base skeleton data
    data: &'static SkeletalData,

    // Internal bone structure
    bones: Vec<Bone>,

    // Lookup table for Name -> Index relation
    name_to_index: HashMap<String, usize>,

    // Iteration indices
    child_first_indices: Vec<usize>,
    child_last_indices: Vec<usize>,

    // World position offset
    local_transform: Vec2,
    world_position: Vec2,
    bounds: (Vec2, Vec2),

    // Animation offsets, with rest angles as defaults
    bone_rest_angles: Vec<AnimationFrameBone>,

    // Animation data
    animator: Animator,

    // Ragdoll
    ragdoll: Option<Ragdoll>

}

impl Skeleton {

    pub fn new(data: &'static SkeletalData) -> Self {

        // Internal Data Structures
        let bones = data.to_internal_bones();

        // Lookups
        let root = vec![0];
        let mut name_to_index = HashMap::with_capacity(bones.len());
        let mut child_first_indices: Vec<usize> = Vec::with_capacity(bones.len());
        let mut child_last_indices: Vec<usize> = Vec::with_capacity(bones.len());

        for b in &bones {
            name_to_index.insert(b.name().to_string(), b.index);
        }

        Skeleton::visit_bones(&bones[..], &root[..], &mut |bone| {
            child_first_indices.push(bone.index);

        }, true);

        Skeleton::visit_bones(&bones[..], &root[..], &mut |bone| {
            child_last_indices.push(bone.index);

        }, false);

        Self {
            // Data Structures
            data: data,
            bones: bones,
            name_to_index: name_to_index,
            child_first_indices: child_first_indices,
            child_last_indices: child_last_indices,

            // Positions
            local_transform: Vec2::new(1.0, 1.0),
            world_position: Vec2::zero(),
            bounds: (Vec2::zero(), Vec2::zero()),

            // Animations
            bone_rest_angles: data.to_animation_bones(),
            animator: AnimatorBuilder::new().build(),

            // Ragdoll
            ragdoll: None

        }

    }


    // Ragdoll ----------------------------------------------------------------
    pub fn at_rest(&self) -> bool {
        if let Some(ref ragdoll) = self.ragdoll {
            ragdoll.at_rest()

        } else {
            false
        }
    }

    pub fn has_ragdoll(&self) -> bool {
        self.ragdoll.is_some()
    }

    pub fn start_ragdoll(&mut self) {

        let particles = self.bones.iter().map(|bone| {
            bone.to_particle(self.local_transform)

        }).collect();

        let mut constraints: Vec<Box<Constraint>> = self.bones.iter().filter_map(|bone| {
            bone.to_constaint()

        }).collect();

        // Additional skeletal constraints
        for constraint in &self.data.constraints {
            match *constraint {
                SkeletalConstraint::Stick(parent, child) => {
                    let parent = self.bone_by_name(parent).unwrap().index();
                    let child = self.bone_by_name(child).unwrap().index();
                    let ap = self.bones[parent].end();
                    let bp = self.bones[child].end();
                    constraints.push(
                        Box::new(StickConstraint::new(
                            format!("s-{}-{}", parent, child),
                            parent,
                            child,
                            (ap - bp).length()
                        ))
                    );
                },
                SkeletalConstraint::Angular(parent, joint, child, left, right) => {
                    let parent = self.bone_by_name(parent).unwrap().index();
                    let joint = self.bone_by_name(joint).unwrap().index();
                    let child = self.bone_by_name(child).unwrap().index();

                    let (left, right) = if f32_equals(self.local_transform.x.signum(), -1.0) {
                        (right, left)

                    } else {
                        (left, right)
                    };

                    let a = self.bones[child].length();
                    let b = self.bones[joint].length();

                    let rest_length = (a * a + b * b - 2.0 * a * b * left.cos()).sqrt();
                    constraints.push(
                        Box::new(AngularConstraint::new(
                            format!("a-{}-{}-{}", parent, joint, child),
                            parent,
                            child,
                            joint,
                            rest_length,
                            true
                        ))
                    );

                    let rest_length = (a * a + b * b - 2.0 * a * b * right.cos()).sqrt();
                    constraints.push(
                        Box::new(AngularConstraint::new(
                            format!("a-{}-{}-{}", parent, joint, child),
                            parent,
                            child,
                            joint,
                            rest_length,
                            false
                        ))
                    );

                }
            }
        }

        let mut ragdoll = Ragdoll::new(particles, constraints);
        ragdoll.split_bone_from_parent("L.Leg");
        ragdoll.split_bone_from_parent("R.Leg");
        ragdoll.split_bone_from_parent("L.Arm");
        ragdoll.split_bone_from_parent("R.Arm");
        self.ragdoll = Some(ragdoll);

    }

    pub fn stop_ragdoll(&mut self) {
        self.ragdoll.take();
    }


    // Offsets & Positions ----------------------------------------------------
    pub fn set_local_transform(&mut self, transform: Vec2) {
        if self.ragdoll.is_none() {
            self.local_transform = transform;
        }
    }

    pub fn set_world_offset(&mut self, p: Vec2) {
        if self.ragdoll.is_none() {
            self.world_position = p;
        }
    }

    pub fn local_transform(&self) -> Vec2 {
        self.local_transform
    }

    pub fn world_offset(&self) -> Vec2 {
        self.world_position
    }

    pub fn to_local(&self, w: Vec2) -> Vec2 {
        w - self.world_position
    }

    pub fn to_world(&self, p: Vec2) -> Vec2 {
        p + self.world_position
    }

    pub fn local_bounds(&self) -> (Vec2, Vec2) {
        if let Some(ref ragdoll) = self.ragdoll {
            ragdoll.bounds()

        } else {
            (
                self.bounds.0.scale(self.local_transform),
                self.bounds.1.scale(self.local_transform)
            )
        }
    }

    pub fn world_bounds(&self) -> (Vec2, Vec2) {
        let bounds = self.local_bounds();
        (
            bounds.0 + self.world_position,
            bounds.1 + self.world_position
        )
    }

    // Updating ---------------------------------------------------------------
    pub fn step<C: Fn(&mut Particle)>(&mut self, dt: f32, gravity: Vec2, collider: C) {

        if let Some(ref mut ragdoll) = self.ragdoll {
            ragdoll.step(dt, gravity, collider);

        } else {

            // Reset bounds
            self.bounds.0.x = 10000.0;
            self.bounds.0.y = 10000.0;
            self.bounds.1.x = -10000.0;
            self.bounds.1.y = -10000.0;

            // Reset animation rest angles
            self.data.reset_animation_bones(&mut self.bone_rest_angles[..]);

            // Forward animations and calculate animation bone angles
            self.animator.update(dt, &mut self.bone_rest_angles[..]);

            // Reset all bones to the base skeleton angles
            for i in &self.child_last_indices {
                let bone = &mut self.bones[*i];
                bone.angle = self.bone_rest_angles[*i].1;
            }

            // Update all bones relative to their parents
            for i in &self.child_last_indices {
                let values = self.calculate_bone(*i);
                let mut bone = &mut self.bones[*i];
                self.bounds.0.x = self.bounds.0.x.min(bone.start.x).min(bone.end.x);
                self.bounds.0.y = self.bounds.0.y.min(bone.start.y).min(bone.end.y);
                self.bounds.1.x = self.bounds.1.x.max(bone.start.x).max(bone.end.x);
                self.bounds.1.y = self.bounds.1.y.max(bone.start.y).max(bone.end.y);
                bone.set(values);
            }

        }

    }


    // Animations -------------------------------------------------------------
    pub fn animator(&mut self) -> &mut Animator {
        &mut self.animator
    }

    pub fn set_animator(&mut self, animator: Animator) {
        self.animator = animator;
    }

    pub fn apply_world_force(&mut self, origin: Vec2, force: Vec2, width: f32) {
        let origin = self.to_local(origin);
        if let Some(ref mut ragdoll) = self.ragdoll {
            ragdoll.apply_force(origin, force, width);
        }
    }

    pub fn apply_local_force(&mut self, origin: Vec2, force: Vec2, width: f32) {
        if let Some(ref mut ragdoll) = self.ragdoll {
            ragdoll.apply_force(origin, force, width);
        }
    }


    // Bones ------------------------------------------------------------------
    pub fn bone_start(&self, space: Space, name: &str) -> Vec2 {
        if let Some(ref ragdoll) = self.ragdoll {
            let start = ragdoll.constraint_points(name).1;
            match space {
                Space::World => self.to_world(start),
                Space::Local => start,
                Space::Animation => start.scale(self.local_transform)
            }

        } else if let Some(bone) = self.bone_by_name(name) {
            let start = bone.start();
            match space {
                Space::World => self.to_world(start.scale(self.local_transform)),
                Space::Local => start.scale(self.local_transform),
                Space::Animation => start
            }

        } else {
            let start = Vec2::zero();
            match space {
                Space::World => self.to_world(start),
                Space::Local | Space::Animation => start
            }
        }
    }

    pub fn bone_end(&self, space: Space, name: &str) -> Vec2 {
        if let Some(ref ragdoll) = self.ragdoll {
            let end = ragdoll.constraint_points(name).0;
            match space {
                Space::World => self.to_world(end),
                Space::Local => end,
                Space::Animation => end.scale(self.local_transform)
            }

        } else if let Some(bone) = self.bone_by_name(name) {
            let end = bone.end();
            match space {
                Space::World => self.to_world(end.scale(self.local_transform)),
                Space::Local => end.scale(self.local_transform),
                Space::Animation => end
            }

        } else {
            let end = Vec2::zero();
            match space {
                Space::World => self.to_world(end),
                Space::Local | Space::Animation => end
            }
        }
    }

    pub fn apply_bone_ik(&mut self, name: &str, mut target: Vec2, positive: bool, transformed: bool) {

        // Ignore setting IKs during ragdoll
        if self.ragdoll.is_some() {
            return;
        }

        // Transform IK target into animation space
        if transformed {
            target = target.scale(self.local_transform);
        }

        // TODO replace IK with angular constraints?
        let (l1, l2, parent, index, origin, parent_rest_angle) = {
            let bone = self.bone_by_name(name).unwrap();
            (
                self.bones[bone.parent].length(),
                bone.length(),
                bone.parent,
                bone.index,
                self.bones[bone.parent].start(),
                // Parent angle offset after animation
                self.bone_rest_angles[bone.parent].1 - self.bones[bone.parent].animation_angle
            )
        };

        if let Some((a1, a2)) = solve_bone_ik(!positive, l1, l2, target.x - origin.x, target.y - origin.y) {

            self.bones[parent].angle = a1 + parent_rest_angle;
            self.bones[index].angle = a2;

            let values = self.calculate_bone(parent);
            self.bones[parent].set_ik(values);

            let values = self.calculate_bone(index);
            self.bones[index].set_ik(values);

        }

    }

    /*
    pub fn apply_bone_ik_new(&mut self, mut target: Vec2, bone: &str, root: &str, transformed: bool) {

        // Ignore setting IKs during ragdoll
        if self.ragdoll.is_some() {
            return;

        // Transform IK target into animation space
        } else if transformed {
            target = target.scale(self.local_transform);
        }

        let bone = self.bone_by_name(bone).unwrap().index();
        let root = self.bone_by_name(root).unwrap().index();

        ccd_ik(target, bone, root, &mut self.bones[..], 3);

    }*/

    pub fn apply_bone_angle(&mut self, name: &str, angle: f32) {
        if let Some(index) = self.name_to_index.get(name) {
            self.bones[*index].set_angle(angle);
        }
    }

    pub fn visit<C: FnMut(Vec2, Vec2, &str)>(&mut self, mut callback: C, children_first: bool) {

        if let Some(ref ragdoll) = self.ragdoll {
            ragdoll.visit(callback);

        } else {

            let sequence = if children_first {
                &self.child_first_indices

            } else {
                &self.child_last_indices
            };

            for i in sequence {
                let bone = &self.bones[*i];
                let start = bone.start().scale(self.local_transform);
                let end = bone.end().scale(self.local_transform);
                callback(start, end, bone.name());
            }

        }

    }


    // Internal ---------------------------------------------------------------
    fn bone_by_name(&self, name: &str) -> Option<&Bone> {
        if let Some(index) = self.name_to_index.get(name) {
            Some(&self.bones[*index])

        } else {
            None
        }
    }

    fn visit_bones<C: FnMut(&Bone)>(
        bones: &[Bone],
        indices: &[usize],
        callback: &mut C,
        children_first: bool

    ) {
        for i in indices {
            let bone = &bones[*i];
            let children = &bone.children[..];
            if children_first {
                Skeleton::visit_bones(bones, children, callback, children_first);
                callback(bone);

            } else {
                callback(bone);
                Skeleton::visit_bones(bones, children, callback, children_first);
            }
        }
    }

    fn calculate_bone(&self, index: usize) -> (f32, Vec2, Vec2) {

        // Compute temporary update angle
        let bone_angle = {

            let bone = &self.bones[index];

            // Get bone's parent's angle
            let parent_angle = if bone.parent == 255 {
                0.0

            } else {
                self.bones[bone.parent].angle
            };

            parent_angle + bone.angle + bone.offset_angle

        };

        let bone = &self.bones[index];

        // Get starting offset from bone's parent
        let start = if bone.parent == 255 {
            Vec2::zero()

        } else {
            self.bones[bone.parent].end()
        };

        // Calculate end offset from angle and length
        let end = if bone.length() > 0.0 {
            start + Angle::offset(bone_angle, bone.length())

        } else {
            start
        };

        (bone_angle, start, end)

    }

}


// Bone Abstraction -----------------------------------------------------------
#[derive(Debug)]
pub struct Bone {
    index: usize,
    parent: usize,
    ragdoll_parent: usize,
    children: Vec<usize>,

    angle: f32,
    animation_angle: f32,
    offset_angle: f32,

    start: Vec2,
    end: Vec2,

    min_angle: Option<f32>,
    max_angle: Option<f32>,

    data: &'static SkeletalBone
}

impl Bone {

    pub fn name(&self) -> &str {
        self.data.0
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn length(&self) -> f32 {
        (self.data.1).1
    }

    pub fn set_angle(&mut self, r: f32) {
        self.offset_angle = r;
    }

    // Internal ---------------------------------------------------------------
    fn to_constaint(&self) -> Option<Box<Constraint>> {
        if self.ragdoll_parent != 255 {
            let mut c = StickConstraint::new(
                self.name().to_string(),
                self.index,
                self.ragdoll_parent,
                self.length()
            );
            c.set_visual(true);
            Some(Box::new(c))

        } else {
            None
        }
    }

    fn to_particle(&self, transform: Vec2) -> Particle {
        Particle::with_inv_mass(self.end().scale(transform), (self.data.1).3)
    }

    fn set(&mut self, values: (f32, Vec2, Vec2)) {
        self.angle = values.0;
        self.animation_angle = values.0;
        self.start = values.1;
        self.end = values.2;
    }

    fn set_ik(&mut self, values: (f32, Vec2, Vec2)) {
        self.angle = values.0;
        self.start = values.1;
        self.end = values.2;
    }

    fn start(&self) -> Vec2 {
        self.start
    }

    fn end(&self) -> Vec2 {
        self.end
    }

    /*
    fn set_start_ik(&mut self, p: Vec2) {
        self.start = p;
        self.end = self.end_computed();
    }

    fn end_computed(&self) -> Vec2 {
        Vec2::new(
            self.start.x + self.angle.cos() * self.length(),
            self.start.y + self.angle.sin() * self.length()
        )
    }*/

}


// Helpers --------------------------------------------------------------------
fn solve_bone_ik(solve_positive: bool, l1: f32, l2: f32, x: f32, y: f32) -> Option<(f32, f32)> {

    let mut found_valid_solution = true;
    let target_dist = x * x + y * y;

    // Compute a new value for a2 along with its cosine
    let cos_angle_2_denom = 2.0 * l1 * l2;
    let (cos_angle_2, sin_angle_2, a2) = if cos_angle_2_denom > EPSILON {

        let mut cos_angle_2 = (target_dist - l1 * l1 - l2 * l2) / cos_angle_2_denom;

        // if our result is not in the legal cosine range, we can not find a
        // legal solution for the target
        if cos_angle_2 < -1.0 || cos_angle_2 > 1.0 {
            found_valid_solution = false;
        }

        // clamp our value into range so we can calculate the best
        // solution when there are no valid ones
        cos_angle_2 = cos_angle_2.min(1.0).max(-1.0);

        // compute a new value for a2
        let mut a2 = cos_angle_2.acos();

        // adjust for the desired bend direction
        if !solve_positive {
            a2 = -a2;
        }

        // compute the sine of our angle
        (cos_angle_2, a2.sin(), a2)

    } else {

        // At least one of the bones had a zero length. This means our
        // solvable domain is a circle around the origin with a radius
        // equal to the sum of our bone lengths.
        let total_len = (l1 + l2) * (l1 + l2);
        if target_dist < total_len - EPSILON || target_dist > total_len + EPSILON {
            found_valid_solution = false;
        }

        // Only the value of angle1 matters at this point. We can just
        // set a2 to zero.
        (1.0, 0.0, 0.0)

    };

    // Compute the value of angle1 based on the sine and cosine of angle2
    let tri_adjacent = l1 + l2 * cos_angle_2;
    let tri_opposite = l2 * sin_angle_2;

    let tan_y = y * tri_adjacent - x * tri_opposite;
    let tan_x = x * tri_adjacent + y * tri_opposite;

    // Note that it is safe to call Atan2(0,0) which will happen if targetX and
    // targetY are zero
    if found_valid_solution {
        Some((tan_y.atan2(tan_x), a2))

    } else {
        None
    }

}

/*
fn ccd_ik(target: Vec2, child: usize, root: usize, bones: &mut[Bone], iterations: usize) {

    // Drag child to target
    {
        let bone = &mut bones[child];
        let delta = target - bone.start();
        bone.angle = delta.y.atan2(delta.x);
        let start = target - Vec2::new(
            bone.angle.cos() * bone.length(),
            bone.angle.sin() * bone.length()
        );
        bone.set_start_ik(start);
    }

    let mut bone_list = Vec::new();
    for iter in 0..iterations {

        // Inverse phase
        let mut current = child;
        loop {

            // Stop in case there are no further parents
            let parent = bones[current].parent;
            if parent == 255 {
                break;
            }

            // Only build bone list during first iteration
            if iter == 0 {
                bone_list.push(bones[current].index());
            }

            // Point parent at child start
            let child_start = bones[current].start();
            let delta = child_start - bones[parent].start();
            let angle = delta.y.atan2(delta.x);
            bones[parent].angle = angle;

            // Stop after root child, we don't want to peposition it
            if parent == root {
                break;
            }

            // Reposition parent
            let length = bones[parent].length();
            bones[parent].set_start_ik(child_start - Vec2::new(
                angle.cos() * length,
                angle.sin() * length
            ));

            // Repeat with next parent
            current = parent;

        }

        // Angle phase
        for i in bone_list.iter() {

            let parent_position = bones[bones[*i].parent].start();
            let bone = &mut bones[*i];

            // Axis of parent and child
            let parent_axis = parent_position - bone.start();
            let child_end = bone.end_computed();
            let child_axis = bone.start() - child_end;

            // Relative angle between parent and child
            let parent_angle = parent_axis.angle();
            let rel_angle = child_axis.angle_between(parent_axis);

            // Clamp angles
            if let Some(min_angle) = bone.min_angle {
                if rel_angle < min_angle {
                    bone.angle = parent_angle + (PI - min_angle);
                    let start = child_end - Vec2::new(
                        bone.angle.cos() * bone.length(),
                        bone.angle.sin() * bone.length()
                    );
                    bone.set_start_ik(start);
                }
            }

            if let Some(max_angle) = bone.max_angle {
                if rel_angle > max_angle {
                    bone.angle = parent_angle + (PI - max_angle);
                    let start = child_end - Vec2::new(
                        bone.angle.cos() * bone.length(),
                        bone.angle.sin() * bone.length()
                    );
                    bone.set_start_ik(start);
               }
            }

        }

        // Forward phase
        for i in bone_list.iter().rev() {
            let start = bones[bones[*i].parent].end_computed();
            bones[*i].set_start_ik(start);
        }

    }

}
*/
