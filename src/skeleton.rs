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
use super::{Angle, Vec2};
use super::animation::{AnimationFrameBone, AnimationData, AnimationBlender};
use super::{
    Constraint, AngularConstraint, StickConstraint, Ragdoll, Particle
};


// Types ----------------------------------------------------------------------
pub enum SkeletalConstraint {
    Stick(&'static str, &'static str),
    Angular(&'static str, &'static str, &'static str, f32, f32),
}

type SkeletalBoneDescription = (
    // Parent, length, angle, ik_inv_mass, ragdoll_inv_mass
    &'static str, f32, f32, f32, f32
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

                tmp_update_angle: 0.0,
                current_angle: 0.0,
                user_angle: 0.0,

                start: Vec2::zero(),
                end: Vec2::zero(),

                world_position: Vec2::zero(),
                local_transform: Vec2::new(1.0, 1.0),

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
    name_to_index: HashMap<&'static str, usize>,

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
    animation: AnimationBlender,

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
            name_to_index.insert(b.name(), b.index);
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
            animation: AnimationBlender::new(),

            // Ragdoll
            ragdoll: None

        }

    }


    // Ragdolls ---------------------------------------------------------------
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

        let particles = self.get_particles();
        let mut constraints = self.get_constraints();

        // Additional skeletal constraints
        for constraint in &self.data.constraints {
            match *constraint {
                SkeletalConstraint::Stick(parent, child) => {
                    let parent = self.get_bone(parent).unwrap().index();
                    let child = self.get_bone(child).unwrap().index();
                    let ap = self.get_bone_index(parent).end_local();
                    let bp = self.get_bone_index(child).end_local();
                    constraints.push((
                        "".to_string(),
                        Box::new(StickConstraint::new(parent, child, (ap - bp).len()))
                    ));
                },
                SkeletalConstraint::Angular(parent, joint, child, left, right) => {
                    let parent = self.get_bone(parent).unwrap().index();
                    let joint = self.get_bone(joint).unwrap().index();
                    let child = self.get_bone(child).unwrap().index();

                    let (left, right) = if self.local_transform.x.signum() == -1.0 {
                        (right, left)

                    } else {
                        (left, right)
                    };

                    let a = self.get_bone_index(child).length();
                    let b = self.get_bone_index(joint).length();

                    let rest_length = (a * a + b * b - 2.0 * a * b * left.cos()).sqrt();
                    constraints.push((
                        "".to_string(),
                        Box::new(AngularConstraint::new(parent, child, joint, rest_length, true))
                    ));

                    let rest_length = (a * a + b * b - 2.0 * a * b * right.cos()).sqrt();
                    constraints.push((
                        "".to_string(),
                        Box::new(AngularConstraint::new(parent, child, joint, rest_length, false))
                    ));

                }
            }
        }

        self.ragdoll = Some(Ragdoll::new(particles, constraints));

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
            self.animation.update(dt, &mut self.bone_rest_angles[..]);

            // Reset all bones to the base skeleton angles
            for i in &self.child_last_indices {
                let bone = &mut self.bones[*i];
                bone.tmp_update_angle = 0.0;
                bone.current_angle = self.bone_rest_angles[*i].1;
            }

            // Update all bones relative to their parents
            for i in &self.child_last_indices {
                let values = self.calculate_bone(*i);
                self.bones[*i].set(values);
            }

        }

    }

    pub fn apply_ik(&mut self, name: &'static str, mut target: Vec2, positive: bool, transformed: bool) {

        // Ignore setting IKs during ragdoll
        if self.ragdoll.is_some() {
            return;
        }

        if transformed {
            target = target.scale(self.local_transform);
        }

        // TODO replace IK with angular constraints?
        let (l1, l2, parent, index, origin, ca) = {
            let bone = self.get_bone(name).unwrap();
            (
                self.bones[bone.parent].length(),
                bone.length(),
                bone.parent,
                bone.index,
                self.bones[bone.parent].start_local(),
                // We need to incorporate the parent bone angle
                // As this does not correctly update with the calculate_bone()
                // calls below
                self.bones[bone.parent].tmp_update_angle
            )
        };

        // FIXME: Currently applying IK twice breaks everything since tmp_update_angle is required
        // in the calculation
        if let Some((a1, a2)) = solve_bone_ik(!positive, l1, l2, target.x - origin.x, target.y - origin.y) {

            // Rest angles are ignored by the IK so we need to add them back in
            self.bones[parent].current_angle = a1 + self.bone_rest_angles[parent].1 - ca;
            self.bones[index].current_angle = a2;

            let values = self.calculate_bone(parent);
            self.bones[parent].set(values);

            let values = self.calculate_bone(index);
            self.bones[index].set(values);

        }

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


    // Updating & Animation ---------------------------------------------------
    pub fn set_animation(
        &mut self,
        data: &'static AnimationData,
        speed_factor: f32,
        blend_duration: f32
    ) {
        self.animation.set(data, blend_duration, speed_factor);
    }


    // Visitor ----------------------------------------------------------------
    pub fn get_bone_end_world(&self, name: &'static str) -> Vec2 {
        self.to_world(self.get_bone_end_local(name))
    }

    pub fn get_bone_start_world(&self, name: &'static str) -> Vec2 {
        self.to_world(self.get_bone_start_local(name))
    }

    pub fn get_bone_end_local(&self, name: &'static str) -> Vec2 {
        self.get_bone_end_ik(name).scale(self.local_transform)
    }

    pub fn get_bone_start_local(&self, name: &'static str) -> Vec2 {
        self.get_bone_start_ik(name).scale(self.local_transform)
    }

    pub fn get_bone_end_ik(&self, name: &'static str) -> Vec2 {
        if let Some(ref ragdoll) = self.ragdoll {
            // TODO we double scale here...
            ragdoll.constraint_points(name).0.scale(self.local_transform)

        } else if let Some(bone) = self.get_bone(name) {
            bone.end_local()

        } else {
            self.world_position
        }
    }

    pub fn get_bone_start_ik(&self, name: &'static str) -> Vec2 {
        if let Some(ref ragdoll) = self.ragdoll {
            // TODO we double scale here...
            ragdoll.constraint_points(name).1.scale(self.local_transform)

        } else if let Some(bone) = self.get_bone(name) {
            bone.start_local()

        } else {
            self.world_position
        }
    }

    pub fn get_bone_mut(&mut self, name: &'static str) -> Option<&mut Bone> {
        if let Some(index) = self.name_to_index.get(name) {
            Some(&mut self.bones[*index])

        } else {
            None
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
                {
                    let b = &mut self.bones[*i];

                    // TODO still needed?
                    b.world_position = self.world_position;
                    b.local_transform = self.local_transform;

                    self.bounds.0.x = self.bounds.0.x.min(b.start.x).min(b.end.x);
                    self.bounds.0.y = self.bounds.0.y.min(b.start.y).min(b.end.y);
                    self.bounds.1.x = self.bounds.1.x.max(b.start.x).max(b.end.x);
                    self.bounds.1.y = self.bounds.1.y.max(b.start.y).max(b.end.y);

                }
                let bone = &self.bones[*i];
                let start = bone.start_local().scale(self.local_transform);
                let end = bone.end_local().scale(self.local_transform);
                callback(start, end, bone.name());
            }

        }

    }


    // Internal ---------------------------------------------------------------
    fn get_bone_index(&self, index: usize) -> &Bone {
        &self.bones[index]
    }

    fn get_bone(&self, name: &'static str) -> Option<&Bone> {
        if let Some(index) = self.name_to_index.get(name) {
            Some(&self.bones[*index])

        } else {
            None
        }
    }

    fn get_particles(&self) -> Vec<Particle> {
        self.bones.iter().map(|bone| {
            bone.to_particle()

        }).collect()
    }

    fn get_constraints(&self) -> Vec<(String, Box<Constraint>)> {
        self.bones.iter().filter_map(|bone| {
            bone.to_constaint()

        }).collect()
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

            // Get bone's parent's temporary update angle
            let parent_angle = if bone.parent == 255 {
                0.0

            } else {
                self.bones[bone.parent].tmp_update_angle
            };

            parent_angle + bone.current_angle + bone.user_angle

        };

        let bone = &self.bones[index];

        // Get starting offset from bone's parent
        let start = if bone.parent == 255 {
            Vec2::zero()

        } else {
            self.bones[bone.parent].end_local()
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

    tmp_update_angle: f32,
    current_angle: f32,
    user_angle: f32,

    start: Vec2, // parent.end
    end: Vec2,

    // Note: Only updated in skeleton visit_*() methods
    world_position: Vec2,
    local_transform: Vec2,

    data: &'static SkeletalBone
}

impl Bone {

    pub fn name(&self) -> &'static str {
        self.data.0
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn ragdoll_parent(&self) -> Option<usize> {
        if self.ragdoll_parent != 255 {
            Some(self.ragdoll_parent)

        } else {
            None
        }
    }

    pub fn parent(&self) -> Option<usize> {
        if self.parent != 255 {
            Some(self.parent)

        } else {
            None
        }
    }

    pub fn start_local(&self) -> Vec2 {
        self.start
    }

    pub fn end_local(&self) -> Vec2 {
        self.end
    }

    pub fn transform(&self, p: Vec2) -> Vec2 {
        p.scale(self.local_transform)
    }

    pub fn length(&self) -> f32 {
        (self.data.1).1
    }

    pub fn set_user_angle(&mut self, r: f32) {
        self.user_angle = r;
    }

    pub fn set_from_ragdoll(&mut self, start: Vec2, end: Vec2) {
        self.start = start;
        self.set_end(end);
    }


    // Internal ---------------------------------------------------------------
    fn to_constaint(&self) -> Option<(String, Box<Constraint>)> {
        if self.ragdoll_parent != 255 {
            let mut c = StickConstraint::new(self.index, self.ragdoll_parent, self.length());
            c.set_visual(true);
            Some((self.name().to_string(), Box::new(c)))

        } else {
            None
        }
    }

    fn to_particle(&self) -> Particle {
        Particle::with_inv_mass(self.transform(self.end_local()), (self.data.1).4)
    }

    fn set(&mut self, values: (f32, Vec2, Vec2)) {
        self.tmp_update_angle = values.0;
        self.start = values.1;
        self.set_end(values.2);
    }

    fn set_end(&mut self, p: Vec2) {
        self.end = p;
    }

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

