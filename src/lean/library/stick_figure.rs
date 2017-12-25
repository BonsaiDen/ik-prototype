// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use lean::{
    Skeleton, SkeletalData,
    AnimationData,
    Angle, Vec2,
    f32_equals
};

use lean::library::{
    Attachement, Renderer, Collider,
    Scarf, StandardRifle
};


// Statics --------------------------------------------------------------------
const D90: f32 = PI * 0.5;
const D45: f32 = D90 * 0.5;
const D22: f32 = D45 * 0.5;
const D12: f32 = D22 * 0.5;

lazy_static! {

    static ref DEFAULT_FIGURE_SKELETON: SkeletalData = SkeletalData {
        bones: vec![
            (  "Root", ( "Root",  0.0, -D90, 0.00, 0.98)), // 0

            (  "Back", ( "Root", 17.0,  0.0, 0.00, 0.99)), // 1
            (  "Neck", ( "Back",  2.0,  0.0, 0.00, 1.00)), // 2
            (  "Head", ( "Neck",  4.0,  0.0, 0.00, 0.99)), // 3

            ( "R.Arm", ( "Back",  9.0,  D90, 0.00, 1.00)), // 6
            ("R.Hand", ("R.Arm", 13.0,  0.0, 0.00, 1.00)), // 7
            ( "L.Arm", ( "Back",  9.0, -D90, 0.00, 1.00)),  // 4
            ("L.Hand", ("L.Arm", 13.0,  0.0, 0.00, 1.00)), // 5

            (  "Hip", ( "Root",   1.0,   PI, 0.00, 1.00)), // 8

            ( "L.Leg", (  "Hip", 13.0,  0.0, 0.00, 1.00)), // 9
            ("L.Foot", ("L.Leg", 14.0,  0.0, 0.00, 1.00)), // 10
            ( "R.Leg", (  "Hip", 13.0,  0.0, 0.00, 1.00)), // 11
            ("R.Foot", ("R.Leg", 14.0,  0.0, 0.00, 1.00)) // 12
        ],
        constraints: vec![
            ("Back", "L.Leg"),
            ("Back", "R.Leg"),

            ("Head", "L.Leg"),
            ("Head", "R.Leg"),

            ("Hip", "L.Arm"),
            ("Hip", "R.Arm")
        ]
    };

    static ref IDLE_ANIMATION: AnimationData = AnimationData {
        name: "Idle",
        duration: 6.25,
        key_frames: vec![
            // Pose
            (0.0, vec![
                ( "L.Leg", -D22),
                ("L.Foot",  0.0),
                ( "R.Leg",  D22),
                ("R.Foot",  0.0),

                ( "R.Arm", D90 * 1.25),
                ("R.Hand",  -D90),
                ( "L.Arm",  -D90 * 0.5),
                ("L.Hand",  -D45 * 1.65)
            ]),
            // Matches the idle compression
            (6.25 * 0.5, vec![
                ( "L.Leg", -D22),
                ("L.Foot",  0.0),
                ( "R.Leg",  D22),
                ("R.Foot",  0.0),

                ( "R.Arm", D90 * 1.0),
                ("R.Hand",  -D90),
                ( "L.Arm",  -D90 * 0.75),
                ("L.Hand",  -D45 * 1.65)

            ])
        ]
    };

    static ref JUMP_ANIMATION: AnimationData = AnimationData {
        name: "Jump",
        duration: 2.0,
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
            (1.0, vec![
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
        name: "Run",
        duration: 10.0,
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
            (3.0, vec![
                ( "L.Leg",  -D45 * 0.95),
                ("L.Foot",  D12 * 0.5),
                ( "R.Leg",  D45),
                ("R.Foot",  D45 * 1.35),

                ( "R.Arm", D90 * 1.95),
                ("R.Hand",  -D90 * 0.90),

                ( "L.Arm", -D90 * 1.45),
                ("L.Hand",  -D90 * 0.90),

            ]),
            // // // Pass with Legs Swapped
            (5.0, vec![
                ( "R.Leg", -D45 * 1.15),
                ("R.Foot",  D45 * 1.95),
                ( "L.Leg", -D12),
                ("L.Foot",  D45),

                ( "R.Arm", D90 * 1.25), // TODO slightly adjust
                ("R.Hand",  -D90 * 0.90),
                ( "L.Arm", -D90 * 0.75), // TODO slightly adjust
                ("L.Hand",  -D90 * 0.90),

            ]),
            // // // Reach Mirrored
            (8.0, vec![
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

    static ref RUN_BACKWARDS_ANIMATION: AnimationData = AnimationData {
        name: "RunBackwards",
        duration: 10.0,
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
            (3.0, vec![
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
            (5.0, vec![
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
            (8.0, vec![
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
    attachements: Vec<Box<Attachement<R, C>>>,

    // Visual feedback
    ragdoll_timer: f32

}

impl<T: StickFigureState, R: Renderer, C: Collider> StickFigure<T, R, C> {

    pub fn default(state: T, config: StickFigureConfig) -> Self {
        StickFigure::from_skeleton(&DEFAULT_FIGURE_SKELETON, state, config)
    }

    pub fn from_skeleton(
        data: &'static SkeletalData,
        state: T,
        config: StickFigureConfig

    ) -> Self {
        Self {
            config: config,
            state: state,

            skeleton: Skeleton::new(data),
            crouch_timer: 0.0,
            idle_timer: 0.0,
            run_timer: 0.0,
            compression_timer: 0.0,
            recoil: 0.0,
            compression: 0.0,
            was_firing: false,
            was_grounded: false,

            ragdoll_timer: 0.0,

            attachements: vec![
                Box::new(Scarf::new(24.0, 6)),
                Box::new(StandardRifle::new())
            ]
        }
    }

    pub fn world_offset(&self) -> Vec2 {
        self.skeleton.world_offset()
    }

    pub fn set_state(&mut self, state: T) {

        self.state = state;

        if !self.state.is_alive() && !self.skeleton.has_ragdoll() {

            let facing = Angle::facing(self.state.direction() + D90).to_vec();
            let force = Vec2::new(-16.0, -31.0).scale(facing);

            // Update weapon model to support ragdoll
            for attachement in &mut self.attachements {
                attachement.loosen(&self.skeleton);
                attachement.apply_force(force * 0.5);
            }

            // Setup skeleton ragdoll
            self.skeleton.start_ragdoll();
            self.skeleton.apply_local_force(Vec2::new(0.0, -10.0), force, 2.0);
            self.ragdoll_timer = 0.0;

        } else if self.state.is_alive() && self.skeleton.has_ragdoll() {
            for attachement in &mut self.attachements {
                attachement.fasten(&self.skeleton);
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

        self.skeleton.get_bone_mut("Back").unwrap().set_user_angle(leanback + velocity.x * 0.05 * facing.x);
        self.skeleton.get_bone_mut("Neck").unwrap().set_user_angle(leanback * self.config.leanback_head_factor);

        // Place and update bones
        if !self.state.is_grounded() {
            self.skeleton.set_animation(&JUMP_ANIMATION, (0.3 * velocity.x.abs().max(1.0).min(1.125)), 0.1);

        } else if velocity.x.abs() > 0.5 {
            if f32_equals(velocity.x.signum(), facing.x) {
                self.skeleton.set_animation(&RUN_ANIMATION, 0.1, 0.1);

            } else {
                self.skeleton.set_animation(&RUN_BACKWARDS_ANIMATION, 0.08, 0.05);
            }

        } else {
            self.skeleton.set_animation(&IDLE_ANIMATION, 1.0 / self.config.idle_speed, 0.1);
        }

        // Offsets
        let idle_offset = ((self.idle_timer * self.config.idle_speed).sin() * self.config.idle_compression) as f32 + self.config.idle_compression * 2.0;
        let idle_offset = Vec2::new(0.0, idle_offset * ((self.idle_timer * self.config.idle_speed).min(1.0)));

        let crouch_offset = ((self.crouch_timer * self.config.crouch_speed).sin() * self.config.crouch_compression) as f32 + self.config.crouch_compression * 4.0;
        let crouch_offset = Vec2::new(0.0, crouch_offset * ((self.crouch_timer * self.config.crouch_speed).min(1.0)));
        let compression = Vec2::new(0.0, (self.compression * ((self.compression_timer * self.config.land_speed).min(3.41).sin()).max(0.0)));

        let run_offset = ((self.run_timer * self.config.run_speed).sin() * self.config.run_compression) as f32 + self.config.run_compression * 2.0;
        let run_offset = Vec2::new(0.0, run_offset * ((self.run_timer * self.config.run_speed).min(1.0)));

        self.skeleton.set_world_offset(
            position + self.config.offset + idle_offset + crouch_offset + run_offset + compression
        );

        // Animate and Arrange
        self.skeleton.step(dt, Vec2::new(0.0, self.config.fall_limit * 100.0), |p| {
            if collider.local(&mut p.position) {
                if ragdoll_timer > 1.0 {
                    p.set_invmass(0.5);
                }
            }
        });

        // Attachement IKs
        for attachement in &self.attachements {
            if let Some(iks) = attachement.get_iks(&self.skeleton, direction, -self.recoil) {
                for (bone, p, positive) in iks {
                    self.skeleton.apply_ik(bone, p, positive);
                }
            }
        }

        // Leg IKs
        if self.state.is_grounded() {
            let mut foot_l = self.skeleton.get_bone_end_ik("L.Foot");
            if collider.local(&mut foot_l) {
                self.skeleton.apply_ik("L.Foot", foot_l, false);
            }

            let mut foot_r = self.skeleton.get_bone_end_ik("R.Foot");
            if collider.local(&mut foot_r) {
                self.skeleton.apply_ik("R.Foot", foot_r, false);
            }
        }

        // Draw bones
        self.skeleton.visit(|bone| {

            let line = (
                bone.start_world(),
                bone.end_world()
            );

            let name = bone.name();
            if name == "R.Arm" || name == "R.Hand" || name == "L.Leg" || name == "L.Foot" {
                renderer.draw_line(line.0, line.1, 0x0080_8080);

            } else if name != "Root" && name != "Head" {
                renderer.draw_line(line.0, line.1, 0x00d0_d0d0);
            }

        }, true);

        let head = self.skeleton.get_bone_end_world("Head");
        renderer.draw_circle(head, 4.0, 0x00d0_d0d0);

        // Draw attachments
        for attachement in &mut self.attachements {
            attachement.fixate(&self.skeleton, direction, -self.recoil);
            attachement.set_gravity(Vec2::new(0.0, self.config.fall_limit * 100.0));
            attachement.step(dt, &collider);
            attachement.draw(renderer);
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
        let shoulder = self.skeleton.get_bone_end_local("Back");
        let aim = shoulder + Angle::offset(
            self.state.direction(),
            self.config.line_of_sight_length
        );
        aim.y - shoulder.y
    }

}

