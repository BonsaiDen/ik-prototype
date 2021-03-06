// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;
use std::collections::HashMap;


// Internal Dependencies ------------------------------------------------------
use ::{
    Skeleton, SkeletalData, SkeletalConstraint,
    AnimatorBuilder, AnimationData,
    Angle, Vec2, Space,
    f32_equals
};

use ::library::{Accessory, Renderer, Collider, Weapon};


// Statics --------------------------------------------------------------------
const D90: f32 = PI * 0.5;
const D45: f32 = D90 * 0.5;
const D22: f32 = D45 * 0.5;
const D12: f32 = D22 * 0.5;

lazy_static! {

    static ref DEFAULT_FIGURE_SKELETON: SkeletalData = SkeletalData {
        bones: vec![
            (  "Root", ( "Root",  0.0, -D90, 0.98, None, None)), // 0

            (  "Back", ( "Root", 18.0,  0.0, 0.99, None, None)), // 1
            (  "Head", ( "Back", 10.0,  0.0, 0.99, None, None)), // 2

            ( "R.Arm", ( "Back",  9.0,  D90, 1.00, None, None)), // 3
            ("R.Hand", ("R.Arm", 13.0,  0.0, 1.00, Some(0.0), Some(D90 * 1.9))), // 4
            ( "L.Arm", ( "Back",  9.0, -D90, 1.00, None, None)),  // 5
            ("L.Hand", ("L.Arm", 13.0,  0.0, 1.00, Some(0.0), Some(D90 * 1.9))), // 6

            (   "Hip", ( "Root",   0.0,  PI, 1.00, None, None)), // 7

            ( "R.Leg", (  "Hip", 13.0,  0.0, 0.99, None, None)), // 8
            ("R.Foot", ("R.Leg", 14.0,  0.0, 1.00, Some(-D90 * 1.9), Some(0.0))), // 9
            ( "L.Leg", (  "Hip", 13.0,  0.0, 0.99, None, None)), // 10
            ("L.Foot", ("L.Leg", 14.0,  0.0, 1.00, Some(-D90 * 1.9), Some(0.0))), // 11
        ],
        ragdoll_parents: vec![
            // Skip hip during ragdolls
            ("L.Leg", "Root"),
            ("R.Leg", "Root")
        ],
        constraints: vec![
            SkeletalConstraint::Stick("Back", "L.Leg"),
            SkeletalConstraint::Stick("Back", "R.Leg"),
            // SkeletalConstraint::Stick("Head", "Root"),

            SkeletalConstraint::Angular("Root", "Back", "Head", PI - D45, PI - D45),
            // SkeletalConstraint::Angular("Root", "L.Leg", "L.Foot", PI, PI - D90 * 1.90),
            // SkeletalConstraint::Angular("Root", "R.Leg", "R.Foot", PI, PI - D90 * 1.90),
            // SkeletalConstraint::Angular("Root", "L.Arm", "L.Hand", PI, PI - D90 * 1.5),
            // SkeletalConstraint::Angular("Root", "R.Arm", "R.Hand", PI, PI - D90 * 1.5)

        ]


    };

    static ref IDLE_ANIMATION: AnimationData = AnimationData {
        duration: 1.25,
        key_frames: vec![
            // Pose
            (0.0, vec![
                ( "L.Leg", -D45),
                ("L.Foot",  D45),
                ( "R.Leg",  0.0),
                ("R.Foot",  D45),

                ( "R.Arm", D90 * 1.25),
                ("R.Hand",  -D90),
                ( "L.Arm",  -D90 * 0.5),
                ("L.Hand",  -D45 * 1.65)
            ]),
            // Matches the idle compression
            (1.25 * 0.5, vec![
                ( "L.Leg", -D45),
                ("L.Foot",  D45),
                ( "R.Leg",  0.0),
                ("R.Foot",  D45),

                ( "R.Arm", D90 * 1.0),
                ("R.Hand",  -D90),
                ( "L.Arm",  -D90 * 0.75),
                ("L.Hand",  -D45 * 1.65)
            ])
        ]
    };

    static ref JUMP_ANIMATION: AnimationData = AnimationData {
        duration: 0.6,
        key_frames: vec![
            // 1
            (0.0, vec![
                ( "L.Leg", -D12 * 3.5 + -D22),
                ("L.Foot",  D22 * 5.0),
                ( "R.Leg", -D12 * 5.0 +  D22),
                ("R.Foot",  D22 * 5.0),

                ( "R.Arm",  -D45 * 1.25),
                ("R.Hand",  -D45 * 1.25),

                ( "L.Arm",  D45 * 1.05),
                ("L.Hand",  D45 * 1.05)
            ]),
            // 2
            (0.3, vec![
                ( "R.Leg", -D12 * 5.0 + -D22),
                ("R.Foot",  D22 * 4.0),
                ( "L.Leg", -D12 * 3.5 +  D22),
                ("L.Foot",  D22 * 4.0),

                ( "R.Arm",  -D45 * 0.5),
                ("R.Hand",  -D45 * 0.5),

                ( "L.Arm",  D45 * 0.4),
                ("L.Hand",  D45 * 0.4)
            ]),
        ]
    };

    static ref RUN_ANIMATION: AnimationData = AnimationData {
        duration: 1.0,
        key_frames: vec![
            // Pass
            (0.0, vec![
                ( "L.Leg", -D45 * 1.15),
                ("L.Foot",  D45 * 1.95),
                ( "R.Leg", -D12),
                ("R.Foot",  D45),

                ( "R.Arm", D90 * 1.25),
                ("R.Hand",  -D90 * 0.90),
                ( "L.Arm", -D90 * 0.75),
                ("L.Hand",  -D90 * 0.90),

            ]),
            // Reach
            (0.3, vec![
                ( "L.Leg",  -D45 * 0.95),
                ("L.Foot",  D12 * 0.5),
                ( "R.Leg",  D45),
                ("R.Foot",  D45 * 1.35),

                ( "R.Arm", D90 * 1.95),
                ("R.Hand",  -D90 * 0.90),

                ( "L.Arm", -D90 * 1.45),
                ("L.Hand",  -D90 * 0.90),

            ]),
            // Pass with Legs Swapped
            (0.5, vec![
                ( "R.Leg", -D45 * 1.15),
                ("R.Foot",  D45 * 1.95),
                ( "L.Leg", -D12),
                ("L.Foot",  D45),

                ( "R.Arm", D90 * 1.25), // TODO slightly adjust
                ("R.Hand",  -D90 * 0.90),
                ( "L.Arm", -D90 * 0.75), // TODO slightly adjust
                ("L.Hand",  -D90 * 0.90),

            ]),
            // Reach Mirrored
            (0.8, vec![
                ( "R.Leg",  -D45 * 0.95),
                ("R.Foot",  D12 * 0.5),
                ( "L.Leg",  D45),
                ("L.Foot",  D45 * 1.35),

                ( "R.Arm", D90 * 0.5),
                ("R.Hand",  -D90 * 0.90),
                ( "L.Arm", -D90 * 0.05),
                ("L.Hand",  -D90 * 0.90),
            ])
        ]
    };

    static ref WALK_BACKWARDS_ANIMATION: AnimationData = AnimationData {
        duration: 0.8,
        key_frames: vec![

            // Pass
            (0.0, vec![
                ( "L.Leg", -D12 * 1.25),
                ("L.Foot", D45 * 1.5),
                ( "R.Leg", D12 * 0.25),
                ("R.Foot", D22 * 0.25),

                ( "R.Arm", D90 * 1.25),
                ("R.Hand",  -D90 * 0.40),
                ( "L.Arm", -D90 * 0.85),
                ("L.Hand",  -D90 * 0.50)
            ]),

            // Reach
            (0.24, vec![
                ( "L.Leg", -D22 * 1.25),
                ("L.Foot", D12),
                ( "R.Leg", D12),
                ("R.Foot", D22),

                ( "R.Arm", D90 * 1.50),
                ("R.Hand",  -D90 * 0.50),
                ( "L.Arm", -D90 * 1.15),
                ("L.Hand",  -D90 * 0.50)
            ]),

            // // Pass
            (0.4, vec![
                ( "R.Leg", -D12 * 1.25),
                ("R.Foot", D45 * 1.5),
                ( "L.Leg", D12 * 0.25),
                ("L.Foot", D22 * 0.25),

                ( "R.Arm", D90 * 1.25),
                ("R.Hand",  -D90 * 0.40),
                ( "L.Arm", -D90 * 0.85),
                ("L.Hand",  -D90 * 0.50)
            ]),

            // Reach
            (0.64, vec![
                ( "R.Leg", -D22 * 1.25),
                ("R.Foot", D12),
                ( "L.Leg", D12),
                ("L.Foot", D22),

                ( "R.Arm", D90 * 1.00),
                ("R.Hand",  -D90 * 0.60),

                ( "L.Arm", -D90 * 0.60),
                ("L.Hand",  -D90 * 0.60)

            ])

        ]
    };

}


// Traits ---------------------------------------------------------------------
pub trait StickFigureState {
    fn is_alive(&self) -> bool;
    fn position(&self) -> Vec2;
    fn velocity(&self) -> Vec2;
    fn force(&self) -> Vec2;
    fn direction(&self) -> f32;
    fn is_grounded(&self) -> bool;
    fn is_crouching(&self) -> bool;
    fn is_firing(&self) -> bool;
}


// Configuration --------------------------------------------------------------
#[derive(Clone)]
pub struct StickFigureConfig {

    pub offset: Vec2,
    pub shoulder_height: f32,
    pub line_of_sight_length: f32,

    pub acceleration: f32,
    pub acceleration_max: f32,

    pub velocity_damping: f32,
    pub velocity_backwards_factor: f32,

    pub jump_force: f32,
    pub fall_speed: f32,
    pub fall_limit: f32,

    pub leanback_min: f32,
    pub leanback_max: f32,
    pub leanback_head_factor: f32,

    pub recoil_leanback_factor: f32,
    pub recoil_force: f32,
    pub recoil_damping: f32,

    pub idle_compression: f32,
    pub idle_speed: f32,

    pub land_compression: f32,
    pub land_compression_factor: f32,
    pub land_speed: f32,

    pub run_compression: f32,
    pub run_speed: f32,

    pub crouching_factor: f32,
    pub crouch_compression: f32,
    pub crouch_speed: f32
}


// Stick Figure Abstraction ---------------------------------------------------
pub struct StickFigure<T: StickFigureState, R: Renderer, C: Collider> {

    // State inputs
    state: T,
    config: StickFigureConfig,

    // Rendering Only
    skeleton: Skeleton,
    idle_timer: f32,
    run_timer: f32,
    crouch_timer: f32,
    compression_timer: f32,
    recoil: f32,
    compression: f32,
    was_firing: bool,
    was_grounded: bool,

    // Attachments
    accessories: HashMap<&'static str, Box<Accessory<R, C>>>,

    // Visual feedback
    ragdoll_timer: f32

}

impl<T: StickFigureState, R: Renderer + 'static, C: Collider + 'static> StickFigure<T, R, C> {

    pub fn default(state: T, config: StickFigureConfig) -> Self {
        StickFigure::from_skeleton(&DEFAULT_FIGURE_SKELETON, state, config)
    }

    pub fn from_skeleton(
        data: &'static SkeletalData,
        state: T,
        config: StickFigureConfig

    ) -> Self {

        let animator = AnimatorBuilder::new().with_state("Idle", |s| {
            s.add_animation(&IDLE_ANIMATION);

        }).with_state("Jump", |s| {
            s.add_animation(&JUMP_ANIMATION);

        }).with_state("Run", |s| {
            s.add_animation(&RUN_ANIMATION);

        }).with_state("Back", |s| {
            s.add_animation(&WALK_BACKWARDS_ANIMATION);

        }).with_blend("*", "Back", 0.05)
          .with_blend("*", "Idle", 0.2)
          .with_blend("Jump", "Idle", 0.1)
          .with_blend("Jump", "Back", 0.2)
          .with_default_blend(0.1).build();

        let mut skeleton = Skeleton::new(data);
        skeleton.set_animator(animator);

        Self {
            config: config,
            state: state,

            skeleton: skeleton,
            crouch_timer: 0.0,
            idle_timer: 0.0,
            run_timer: 0.0,
            compression_timer: 0.0,
            recoil: 0.0,
            compression: 0.0,
            was_firing: false,
            was_grounded: false,

            ragdoll_timer: 0.0,

            accessories: HashMap::new()
        }
    }


    // Accessories ------------------------------------------------------------
    pub fn add_accessory<A: Accessory<R, C> + 'static>(
        &mut self,
        name: &'static str,
        bone: &'static str,
        accessory: A
    ) {
        let mut a = Box::new(accessory) as Box<Accessory<R, C>>;
        a.set_bone(bone);
        self.accessories.insert(name, a);
    }

    pub fn remove_accessory(&mut self, name: &'static str) -> Option<Box<Accessory<R, C>>> {
        self.accessories.remove(name)
    }

    pub fn attach(&mut self, name: &'static str) {
        if let Some(accessory) = self.accessories.get_mut(name) {
            accessory.attach(&self.skeleton);
        }
    }

    pub fn detach(&mut self, name: &'static str) {
        if let Some(accessory) = self.accessories.get_mut(name) {
            accessory.detach(&self.skeleton);
        }
    }

    pub fn get_accessory_mut<A: Accessory<R, C>>(&mut self, name: &'static str) -> Option<&mut A> {
        if let Some(a) = self.accessories.get_mut(name) {
            a.downcast_mut::<A>()

        } else {
            None
        }
    }


    // Getters ----------------------------------------------------------------
    pub fn world_bounds(&self) -> (Vec2, Vec2) {
        self.skeleton.world_bounds()
    }

    pub fn world_offset(&self) -> Vec2 {
        self.skeleton.world_offset()
    }

    pub fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    pub fn skeleton_mut(&mut self) -> &mut Skeleton {
        &mut self.skeleton
    }


    // Setters ----------------------------------------------------------------
    pub fn set_state(&mut self, state: T) {

        self.state = state;

        if !self.state.is_alive() && !self.skeleton.has_ragdoll() {

            let facing = Angle::facing(self.state.direction() + D90).to_vec();
            let force = self.state.force().scale(facing) + self.state.velocity();

            // Update weapon model to support ragdoll
            for accessory in self.accessories.values_mut() {
                let was_attached = accessory.attached();
                accessory.detach(&self.skeleton);
                if was_attached {
                    accessory.apply_force(force * 0.5);
                }
            }

            // Setup skeleton ragdoll
            self.skeleton.start_ragdoll();
            self.skeleton.apply_local_force(Vec2::new(0.0, -10.0), force, 2.0);
            self.ragdoll_timer = 0.0;

        } else if self.state.is_alive() && self.skeleton.has_ragdoll() {
            for accessory in self.accessories.values_mut() {
                accessory.attach(&self.skeleton);
            }
            self.skeleton.stop_ragdoll();
        }

    }

    pub fn draw(
        &mut self,
        renderer: &mut R,
        collider: &C
    ) {

        // Update timers
        let dt = renderer.dt();
        self.update(dt);

        // Gather state data
        let direction = self.state.direction();
        let facing = Angle::facing(direction + D90).to_vec();
        let velocity = self.state.velocity();
        let position = self.state.position();
        let ragdoll_timer = self.ragdoll_timer;

        self.skeleton.set_local_transform(facing);

        // Aim Leanback
        let aim_horizon = self.compute_view_horizon_distance();
        let leanback = (
            aim_horizon * 0.5
            - self.recoil * self.config.recoil_leanback_factor

        ).min(self.config.leanback_max).max(self.config.leanback_min) * 0.009;;

        self.skeleton.apply_bone_angle("Back", leanback + velocity.x * 0.05 * facing.x);
        self.skeleton.apply_bone_angle("Head", leanback * self.config.leanback_head_factor);

        // Update Animations
        let run_factor = (1.0 / 3.5 * velocity.x).abs();
        let walk_backwards_factor = (self.config.velocity_backwards_factor / (3.5 * 0.5) * velocity.x).abs();
        if !self.state.is_grounded() {
            self.skeleton.animator().set_speed("Jump", velocity.x.abs().max(1.0).min(1.5));
            self.skeleton.animator().transition_to("Jump");

        } else if velocity.x.abs() > 0.5 {
            if f32_equals(velocity.x.signum(), facing.x) {
                self.skeleton.animator().set_speed("RuN", run_factor);
                self.skeleton.animator().transition_to("Run");

            } else {
                self.skeleton.animator().set_speed("Back", walk_backwards_factor);
                self.skeleton.animator().transition_to("Back");
            }

        } else {
            // TODO add in idle speed for multiplication
            self.skeleton.animator().set_speed("Idle", 1.0);
            self.skeleton.animator().transition_to("Idle");
        }

        // Offsets
        let idle_offset = ((self.idle_timer * self.config.idle_speed).sin() * self.config.idle_compression) as f32 + self.config.idle_compression * 2.0;
        let idle_offset = Vec2::new(0.0, idle_offset * ((self.idle_timer * self.config.idle_speed).min(1.0)));

        let crouch_offset = ((self.crouch_timer * self.config.crouch_speed).sin() * self.config.crouch_compression) as f32 + self.config.crouch_compression * 4.0;
        let crouch_offset = Vec2::new(0.0, crouch_offset * ((self.crouch_timer * self.config.crouch_speed).min(1.0)));
        let compression = Vec2::new(0.0, (self.compression * ((self.compression_timer * self.config.land_speed).min(3.41).sin()).max(0.0)));

        let run_offset = ((self.run_timer * self.config.run_speed).sin() * self.config.run_compression) as f32 + self.config.run_compression * 2.0;
        let run_offset = Vec2::new(0.0, run_offset * ((self.run_timer * self.config.run_speed).min(1.0)));

        let mut recoil_compression = Angle::from_radians(direction).to_unit_vec() * -self.recoil * 0.5;
        recoil_compression.y = recoil_compression.y.max(0.0);

        self.skeleton.set_world_offset(
            position + self.config.offset + idle_offset + crouch_offset + run_offset + compression + recoil_compression
        );

        // Animate and Arrange
        let world_offset = self.skeleton.world_offset();
        self.skeleton.step(dt, Vec2::new(0.0, self.config.fall_limit * 100.0), |p| {
            if let Some((pos, _, vertical)) = collider.world(p.position + world_offset) {
                p.position = pos - world_offset;
                if ragdoll_timer > 1.0 && vertical == 1 {
                    p.set_invmass(0.5);
                }
            }
        });

        // Accessory IKs
        for accessory in self.accessories.values() {
            if let Some(iks) = accessory.get_iks(&self.skeleton) {
                for (bone, p, positive) in iks {
                    self.skeleton.apply_bone_ik(bone, p, positive, false);
                }
            }
        }

        // Leg IKs
        if self.state.is_grounded() {
            let foot_l = self.skeleton.bone_end(Space::Local, "L.Foot");
            if let Some((p, _, _)) = collider.world(foot_l + world_offset) {
                self.skeleton.apply_bone_ik("L.Foot", p - world_offset, false, true);
                //self.skeleton.apply_bone_ik_new(p - world_offset - Vec2::new(0.0, 0.0), "L.Foot", "Hip", true);
            }

            let foot_r = self.skeleton.bone_end(Space::Local, "R.Foot");
            if let Some((p, _, _)) = collider.world(foot_r + world_offset) {
                self.skeleton.apply_bone_ik("R.Foot", p - world_offset, false, true);
                //self.skeleton.apply_bone_ik_new(p - world_offset, "R.Foot", "Hip", true);
            }
        }

        // Draw bones
        self.skeleton.visit(|start, end, name| {

            let line = (
                start + world_offset,
                end + world_offset
            );

            if name == "R.Arm" || name == "R.Hand" || name == "R.Leg" || name == "R.Foot" {
                renderer.draw_line(line.0, line.1, 0x0080_8080);

            } else if name != "Root" && name != "Head" {
                renderer.draw_line(line.0, line.1, 0x00d0_d0d0);
            }

        }, true);

        // Draw Head
        let head_end = self.skeleton.bone_end(Space::World, "Head");
        let head_start = self.skeleton.bone_start(Space::World, "Head");
        let head_offset = (head_end - head_start) * 0.5;
        renderer.draw_circle(head_start + head_offset, 4.0, 0x00d0_d0d0);

        // Special weapon handling
        let recoil = self.recoil;
        if let Some(weapon) = self.get_accessory_mut::<Weapon>("Weapon") {
            weapon.set_aim_direction(direction);
            weapon.set_recoil(recoil);
        }

        // Draw attachments
        for accessory in self.accessories.values_mut() {
            accessory.fixate(&self.skeleton);
            accessory.set_gravity(Vec2::new(0.0, self.config.fall_limit * 100.0));
            accessory.step(renderer, collider);
            accessory.draw(renderer);
        }

    }

    // Internal ---------------------------------------------------------------
    fn update(&mut self, dt: f32) {

        // Update animation timers
        if self.skeleton.has_ragdoll() {
            self.ragdoll_timer += dt;
        }

        if !self.state.is_alive() {
            return;
        }

        let velocity = self.state.velocity();

        // Compression
        if !self.was_grounded && self.state.is_grounded() {
            self.compression_timer = 0.0;
            self.compression = self.config.land_compression;
        }

        if self.state.is_grounded() {
            self.compression_timer += dt;
        }

        self.compression *= self.config.land_compression_factor;

        // Firing
        if !self.was_firing && self.state.is_firing() {
            self.recoil = self.config.recoil_force;

        } else {
            self.recoil *= self.config.recoil_damping;
        }

        // Idling
        if velocity.x == 0.0 && self.state.is_grounded() && !self.state.is_crouching() {
            self.idle_timer += dt;

        } else {
            self.idle_timer = 0.0;
        }

        // Running
        if velocity.x.abs() > 1.0 && self.state.is_grounded() && !self.state.is_crouching() {
            self.run_timer += dt;

        } else {
            self.run_timer = 0.0;
        }

        // Crouching
        if self.state.is_grounded() && self.state.is_crouching() {
            self.crouch_timer += dt;

        } else {
            self.crouch_timer *= 0.9;
        }

        // State change detection
        self.was_firing = self.state.is_firing();
        self.was_grounded = self.state.is_grounded();

    }

    fn compute_view_horizon_distance(&self) -> f32 {
        let shoulder = self.skeleton.bone_end(Space::Local, "Back");
        let aim = shoulder + Angle::offset(
            self.state.direction(),
            self.config.line_of_sight_length
        );
        aim.y - shoulder.y
    }

}

