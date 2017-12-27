#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct Vec2 {
    x: f32,
    y: f32
}

impl Vec2 {

    fn new(x: f32, y: f32) -> Vec2 {
        Vec2 {
            x,
            y
        }
    }
    fn zero() -> Vec2 {
        Vec2 {
            x: 0.0,
            y: 0.0
        }
    }

}


// Types ----------------------------------------------------------------------
type SkeletalBoneDescription = (
    // Parent, length, angle, ik_inv_mass, ragdoll_inv_mass
    &'static str, f32, f32, f32, f32
);
type SkeletalBone = (&'static str, SkeletalBoneDescription);

// Bone Abstraction -----------------------------------------------------------
#[derive(Debug)]
pub struct Bone {
    index: usize,
    parent: usize,
    children: Vec<usize>,

    tmp_update_angle: f32,
    current_angle: f32,
    user_angle: f32,

    length: f32,
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

    pub fn length(&self) -> f32 {
        (self.data.1).1
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn parent(&self) -> Option<usize> {
        if self.parent != 255 {
            Some(self.parent)

        } else {
            None
        }
    }

}

// Skeleton Data Abstraction --------------------------------------------------
pub struct SkeletalData {
    pub bones: Vec<SkeletalBone>
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

            Bone {
                index: index,
                parent: parent,
                children: Vec::new(),

                tmp_update_angle: 0.0,
                current_angle: 0.0,
                user_angle: 0.0,

                start: Vec2::zero(),
                end: Vec2::zero(),

                length: (bone.1).1,
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

}


lazy_static! {
    static ref SKELETON: SkeletalData = SkeletalData {
        bones: vec![
            (  "Root", ( "Root",  0.0, 0.0, 0.00, 0.98)), // 0

            (  "Back", ( "Root", 18.0,  0.0, 0.00, 0.99)), // 1
            (  "Head", ( "Back", 10.0,  0.0, 0.00, 0.99)), // 3

            ( "R.Arm", ( "Back",  9.0,  0.0, 0.00, 1.00)), // 6
            ("R.Hand", ("R.Arm", 13.0,  0.0, 0.00, 1.00)), // 7
            ( "L.Arm", ( "Back",  9.0,  0.0, 0.00, 1.00)),  // 4
            ("L.Hand", ("L.Arm", 13.0,  0.0, 0.00, 1.00)), // 5

            (  "Hip", ( "Root",   0.0,  0.0, 0.00, 1.00)), // 8

            ( "L.Leg", (  "Hip", 13.0,  0.0, 0.00, 0.99)), // 9
            ("L.Foot", ("L.Leg", 14.0,  0.0, 0.00, 1.00)), // 10
            ( "R.Leg", (  "Hip", 13.0,  0.0, 0.00, 0.99)), // 11
            ("R.Foot", ("R.Leg", 14.0,  0.0, 0.00, 1.00)) // 12
        ]
    };
}

fn main() {

    let bones = SKELETON.to_internal_bones();

    for bone in &bones {
        if let Some(parent) = bone.parent() {
            if bones[parent].length() == 0.0 {
                find_next_valid_parent(bone, &bones[..]);
            }
        }
    }

}

fn find_next_valid_parent(bone: &Bone, bones: &[Bone]) {
    println!("\nFind valid parent for: {}", bone.name());

    let original_bone_index = bone.index();
    let original_bone_children = &bone.children[..];
    let mut upward = true;
    let mut bone = &bones[bone.parent().unwrap()];
    while bone.length() == 0.0 {

        if upward {
            println!("  (upward) next parent: {}", bone.name());
            if let Some(parent) = bone.parent() {
                bone = &bones[parent];

            } else {
                upward = false;
            }

        } else if bone.children.is_empty() {
            break;

        } else {
            let mut i = 0;
            loop {
                let p_bone = &bones[bone.children[i]];
                let bone_index = p_bone.index();
                if bone_index != original_bone_index && !original_bone_children.contains(&bone_index) {
                    bone = p_bone;
                    break;
                }
                i += 1;
            }

            println!("  (down) next child: {}", bone.name());
        }
    }

}

