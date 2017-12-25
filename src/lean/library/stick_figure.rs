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
    ParticleSystem, ParticleTemplate,
    RigidBodyData, RigidBody,
    f32_equals
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

            ( "L.Arm", ( "Back",  9.0, -D90, 0.00, 1.00)),  // 4
            ("L.Hand", ("L.Arm", 13.0,  0.0, 0.00, 1.00)), // 5
            ( "R.Arm", ( "Back",  9.0,  D90, 0.00, 1.00)), // 6
            ("R.Hand", ("R.Arm", 13.0,  0.0, 0.00, 1.00)), // 7

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
        duration: 2.0,
        key_frames: vec![
            // Pose
            (0.0, vec![
                ( "L.Leg", -D22),
                ("L.Foot",  0.0),
                ( "R.Leg",  D22),
                ("R.Foot",  0.0)
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
                ("R.Foot",  D22 * 5.0)
            ]),
            // 2
            (1.0, vec![
                ( "R.Leg", -D12 * 5.0 + -D22),
                ("R.Foot",  D22 * 4.0),
                ( "L.Leg", -D12 * 3.5 +  D22),
                ("L.Foot",  D22 * 4.0)
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
                ("R.Foot",  D45)
            ]),
            // Reach
            (3.0, vec![
                ( "L.Leg",  -D45 * 0.95),
                ("L.Foot",  D12 * 0.5),
                ( "R.Leg",  D45),
                ("R.Foot",  D45 * 1.35)
            ]),
            // Pass with Legs Swapped
            (5.0, vec![
                ( "R.Leg", -D45 * 1.15),
                ("R.Foot",  D45 * 1.95),
                ( "L.Leg", -D12),
                ("L.Foot",  D45)
            ]),
            // Reach Mirrored
            (8.0, vec![
                ( "R.Leg",  -D45 * 0.95),
                ("R.Foot",  D12 * 0.5),
                ( "L.Leg",  D45),
                ("L.Foot",  D45 * 1.35)
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
                ("R.Foot", D22 * 0.25)
            ]),

            // Reach
            (3.0, vec![
                ( "L.Leg", -D22 * 1.25),
                ("L.Foot", D12),
                ( "R.Leg", D12),
                ("R.Foot", D22)
            ]),

            // Pass
            (5.0, vec![
                ( "R.Leg", -D12 * 1.25),
                ("R.Foot", D45 * 1.5),
                ( "L.Leg", D12 * 0.25),
                ("L.Foot", D22 * 0.25)
            ]),

            // Reach
            (8.0, vec![
                ( "R.Leg", -D22 * 1.25),
                ("R.Foot", D12),
                ( "L.Leg", D12),
                ("L.Foot", D22)
            ])

        ]
    };

    static ref WEAPON_RIGID: RigidBodyData = RigidBodyData {
        points: vec![
            ("Center", 15.0, 0.0),
            ("Barrel", 30.0, 0.0),
            ("StockMid", 0.0, 0.0),
            ("StockLow", 0.0, 5.0),
        ],
        constraints: vec![
            ("Center", "Barrel", true),
            ("Center", "StockMid", true),
            ("Center", "StockLow", true),
            ("StockMid", "StockLow", true),
            ("StockLow", "Barrel", false)
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

pub trait StickFigureRenderer {
    fn dt(&self) -> f32;
    fn circle_vec(&mut self, c: Vec2, r: f32, color: u32);
    fn line_vec(&mut self, start: Vec2, end: Vec2, color: u32);
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
pub struct StickFigure<T: StickFigureState> {

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

    // Visual feedback
    ragdoll_timer: f32,
    scarf_timer: f32,
    scarf: ParticleSystem,
    weapon: RigidBody

}

impl<T: StickFigureState> StickFigure<T> {

    pub fn default(state: T, config: StickFigureConfig) -> Self {
        StickFigure::from_skeleton(&DEFAULT_FIGURE_SKELETON, state, config)
    }

    pub fn from_skeleton(
        data: &'static SkeletalData,
        state: T,
        config: StickFigureConfig

    ) -> Self {
        let scarf = ParticleTemplate::schal(1, 6, 4.0, Vec2::zero());
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

            scarf_timer: 0.0,
            scarf: scarf,
            weapon: RigidBody::new(&WEAPON_RIGID)
        }
    }

    pub fn to_local(&self, p: Vec2) -> Vec2 {
        self.skeleton.to_local(p)
    }

    pub fn set_state(&mut self, state: T) {

        self.state = state;

        if !self.state.is_alive() && !self.skeleton.has_ragdoll() {

            let facing = Angle::facing(self.state.direction() + D90).to_vec();
            let force = Vec2::new(-16.0, -31.0).scale(facing);

            // Update weapon model to support ragdoll
            self.weapon.make_dynamic();
            self.weapon.apply_dynamic_force(force * 0.5);

            // Setup skeleton ragdoll
            self.skeleton.start_ragdoll();
            self.skeleton.apply_local_force(Vec2::new(0.0, -10.0), force, 2.0);
            self.ragdoll_timer = 0.0;

        } else if self.state.is_alive() && self.skeleton.has_ragdoll() {
            self.scarf.visit_particles_mut(|_, particle| {
                particle.set_position(Vec2::zero());
            });
            self.skeleton.stop_ragdoll();
        }

    }

    pub fn draw<
        R: StickFigureRenderer,
        C: Fn(&mut Vec2) -> bool,
        D: Fn(&mut Vec2) -> bool

    >(&mut self, renderer: &mut R, collider_local: C, collider_world: D) {

        // Update timers
        let dt = renderer.dt();
        self.update(dt);

        // Gather state data
        let facing = Angle::facing(self.state.direction() + D90).to_vec();
        let velocity = self.state.velocity();
        let position = self.state.position();

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
            self.skeleton.set_animation(&JUMP_ANIMATION, (0.3 * velocity.x.abs().max(1.0).min(1.125)), 0.05);

        } else if velocity.x.abs() > 0.5 {
            if f32_equals(velocity.x.signum(), facing.x) {
                self.skeleton.set_animation(&RUN_ANIMATION, 0.1, 0.05);

            } else {
                self.skeleton.set_animation(&RUN_BACKWARDS_ANIMATION, 0.08, 0.05);
            }

        } else {
            self.skeleton.set_animation(&IDLE_ANIMATION, 0.1, 0.05);
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
        let ragdoll_timer = self.ragdoll_timer;
        self.skeleton.step(dt, Vec2::new(0.0, self.config.fall_limit * 100.0), |p| {
            if collider_local(&mut p.position) {
                if ragdoll_timer > 1.0 {
                    p.set_invmass(0.5);
                }
            }
        });

        // Weapon Grip IK
        // TODO abstract scarf and weapon into attachements
        // TODO add IK position settings to weapon instead
        // TODO have a holdable trait or something
        let shoulder = self.skeleton.get_bone_end_ik("Back");
        let grip_angle = Angle::transform(self.state.direction(), facing);
        let grip = shoulder + Angle::offset(grip_angle, 17.0 - self.recoil) + Angle::offset(grip_angle + D90, 1.0);
        let trigger = shoulder + Angle::offset(grip_angle, 6.5 - self.recoil * 0.5) + Angle::offset(grip_angle + D90, 4.0);
        self.skeleton.apply_ik("L.Hand", grip, true);
        self.skeleton.apply_ik("R.Hand", trigger, true);

        // Leg IK
        if self.state.is_grounded() {
            let mut foot_l = self.skeleton.get_bone_end_ik("L.Foot");
            if collider_local(&mut foot_l) {
                self.skeleton.apply_ik("L.Foot", foot_l, false);
            }

            let mut foot_r = self.skeleton.get_bone_end_ik("R.Foot");
            if collider_local(&mut foot_r) {
                self.skeleton.apply_ik("R.Foot", foot_r, false);
            }
        }

        // Draw scarf
        // TODO abstract scarf and weapon into attachements
        let neck = self.skeleton.get_bone_end_local("Neck");
        self.scarf.get_mut(0).set_position(neck);

        self.scarf.activate(); // Don't let the scarf fall into rest
        self.scarf.step(dt, Vec2::new(-200.0 * facing.x, (self.scarf_timer * 4.0).sin() * self.config.fall_limit * 50.0), |p| {
            collider_local(&mut p.position);
        });

        let neck_offset = self.skeleton.get_bone_end_world("Neck") - neck;
        self.scarf.visit_particles_chained(|i, p, n| {
            renderer.line_vec(neck_offset + p.position, neck_offset + n.position, 0x00ff_ff00);
        });

        // Draw bones
        self.skeleton.visit(|bone| {

            let line = (
                bone.start_world(),
                bone.end_world()
            );

            let name = bone.name();
            if name == "Head" {
                renderer.circle_vec(line.1, 4.0, 0x00d0_d0d0);

            } else if name == "L.Arm" || name == "L.Hand" || name == "L.Leg" || name == "L.Foot" {
                renderer.line_vec(line.0, line.1, 0x0080_8080);

            } else if name != "Root" {
                renderer.line_vec(line.0, line.1, 0x00d0_d0d0);
            }

        }, false);

        // Draw Weapon
        // TODO move weapon out?
        // TODO add arm movement to running animation?
        // TODO add arm movement to idle animation?
        if self.skeleton.has_ragdoll() {
            self.weapon.step_dynamic(dt, Vec2::new(0.0, self.config.fall_limit * 100.0), |p| {
                if collider_world(&mut p.position) {
                    if ragdoll_timer > 1.0 {
                        p.set_invmass(0.5);
                    }
                }
            });
            self.weapon.visit_dynamic(|(_, a), (_, b), _| {
                renderer.line_vec(
                    a,
                    b,
                    0x00ff_ff00
                );
            });

        } else {
            let shoulder = self.skeleton.get_bone_end_world("Back");
            self.weapon.step_static(
                shoulder,
                Vec2::new(-self.recoil, 0.0),
                facing.flipped(),
                self.state.direction()
            );

            self.weapon.visit_static(|a, b| {
                renderer.line_vec(
                    a,
                    b,
                    0x00ff_ff00
                );
            });
        }

    }

    // Internal ---------------------------------------------------------------
    fn update(&mut self, dt: f32) {

        // Update animation timers
        if self.skeleton.has_ragdoll() {
            self.ragdoll_timer += dt;
        }
        self.scarf_timer += dt;

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

