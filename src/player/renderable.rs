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
    StickConstraint,
    ParticleSystem, ParticleTemplate,
    RigidBodyData, RigidBody
};

use super::Context;
use super::{Config, PlayerState};
use ::demo::Level;


// Statics --------------------------------------------------------------------
const D90: f32 = PI * 0.5;
const D45: f32 = D90 * 0.5;
const D22: f32 = D45 * 0.5;
const D12: f32 = D22 * 0.5;

lazy_static! {

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

pub struct PlayerRenderable {

    // Shared Logic
    config: Config,
    state: PlayerState,

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
    ragdoll_facing: Vec2,
    ragdoll: Option<ParticleSystem>,
    scarf_timer: f32,
    scarf: ParticleSystem,
    weapon: RigidBody

}

impl PlayerRenderable {

    pub fn from_skeleton(data: &'static SkeletalData, state: PlayerState, config: Config) -> Self {
        let scarf = ParticleTemplate::schal(1, 6, 4.0, state.position);
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
            ragdoll_facing: Vec2::zero(),
            ragdoll: None,

            scarf_timer: 0.0,
            scarf: scarf,
            weapon: RigidBody::new(&WEAPON_RIGID)
        }
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
        if self.state.hp == 0 && self.ragdoll.is_none() {

            let facing = Angle::facing(self.state.direction + D90).to_vec();
            let force = Vec2::new(-16.0, -31.0).scale(facing);

            // Update weapon model to support ragdoll
            self.weapon.make_dynamic();
            self.weapon.apply_dynamic_force(force * 0.5);

            // Create skeleton ragdoll
            self.ragdoll = Some(self.create_ragdoll(force));
            self.ragdoll_timer = 0.0;
            self.ragdoll_facing = facing;

        } else if self.state.hp > 0 && self.ragdoll.is_some() {
            let p = self.state.position;
            self.scarf.visit_particles_mut(|_, particle| {
                particle.set_position(p);
            });
            self.ragdoll.take();
        }
    }

    pub fn update(&mut self, dt: f32) {

        if self.state.hp == 0 {
            return;
        }

        if !self.was_grounded && self.state.is_grounded {
            self.compression_timer = 0.0;
            self.compression = self.config.land_compression;
        }
        if self.state.is_grounded {
            self.compression_timer += dt;
        }

        self.compression *= self.config.land_compression_factor;

        if !self.was_firing && self.state.is_firing {
            self.recoil = self.config.recoil_force;

        } else {
            self.recoil *= self.config.recoil_damping;
        }

        if self.state.velocity.x == 0.0 && self.state.is_grounded && !self.state.is_crouching {
            self.idle_timer += dt;

        } else {
            self.idle_timer = 0.0;
        }

        if self.state.velocity.x.abs() > 1.0 && self.state.is_grounded && !self.state.is_crouching {
            self.run_timer += dt;

        } else {
            self.run_timer = 0.0;
        }

        if self.state.is_grounded && self.state.is_crouching {
            self.crouch_timer += dt;

        } else {
            self.crouch_timer *= 0.9;
        }

        self.was_firing = self.state.is_firing;
        self.was_grounded = self.state.is_grounded;

    }

    pub fn draw(&mut self, context: &mut Context, level: &Level) {

        // Update ragdoll
        let ragdoll_timer = self.ragdoll_timer;

        // TODO merge ragdoll into skeleton
        // TODO we'll need a closure for the time being until all configuration
        // TODO is embedded within the skeletal data structure
        if let Some(ref mut ragdoll) = self.ragdoll {

            self.ragdoll_timer += context.dt();

            let floor = self.skeleton.to_local(Vec2::new(0.0, level.floor));
            ragdoll.step(context.dt(), Vec2::new(0.0, 240.0), |p| {
                if p.position.y > floor.y {
                    if ragdoll_timer > 1.0 {
                        p.set_invmass(0.5);
                    }
                    p.position.y = p.position.y.min(floor.y);
                }
            });

            self.skeleton.set_local_transform(self.ragdoll_facing);
            self.skeleton.visit_mut(|bone| {
                if let Some(parent) = bone.parent() {
                    let index = bone.index();
                    let start = bone.transform(ragdoll.get(parent).position);
                    let end = bone.transform(ragdoll.get(bone.index()).position);
                    bone.set_from_ragdoll(start, end);
                }

            }, false);

        } else {
            self.skeleton.set_local_transform(Angle::facing(self.state.direction + D90).to_vec());
            self.update_skeleton(context.dt(), level);
        };

        let facing = self.skeleton.local_transform();

        // Draw scarf
        let neck = self.skeleton.get_bone_end_world("Neck");
        self.scarf.get_mut(0).set_position(neck);

        self.scarf_timer += context.dt();
        self.scarf.activate(); // Don't let the scarf fall into rest
        self.scarf.step(context.dt(), Vec2::new(-200.0 * facing.x, (self.scarf_timer * 4.0).sin() * 150.0), |p| {
            p.position.y = p.position.y.min(level.floor);
        });

        self.scarf.visit_particles_chained(|_, p, n| {
            context.line_vec(p.position, n.position, 0x00ffff00);
        });

        // Draw bones
        self.skeleton.visit(|bone| {

            let line = (
                bone.start_world(),
                bone.end_world()
            );

            let name = bone.name();
            if name == "Head" {
                context.circle_vec(line.1, 4.0, 0x00d0d0d0);

            } else if name == "L.Arm" || name == "L.Hand" {
                context.line_vec(line.0, line.1, 0x00808080);

            } else if name == "L.Leg" || name == "L.Foot" {
                context.line_vec(line.0, line.1, 0x00808080);

            } else if name != "Root" {
                context.line_vec(line.0, line.1, 0x00d0d0d0);
            }

        }, false);

        // Draw Weapon
        if self.ragdoll.is_none() {
            let shoulder = self.skeleton.get_bone_end_world("Back");
            self.weapon.step_static(
                shoulder,
                Vec2::new(-self.recoil, 0.0),
                facing.flipped(),
                self.state.direction
            );

            self.weapon.visit_static(|a, b| {
                context.line_vec(
                    a,
                    b,
                    0x00ffff00
                );
            });

        } else {
            self.weapon.step_dynamic(context.dt(), Vec2::new(0.0, 240.0), |p| {
                if p.position.y > level.floor {
                    if ragdoll_timer > 1.0 {
                        p.set_invmass(0.5);
                    }
                    p.position.y = p.position.y.min(level.floor);
                }
            });
            self.weapon.visit_dynamic(|(_, a), (_, b), _| {
                context.line_vec(
                    a,
                    b,
                    0x00ffff00
                );
            });
        }

    }

    fn create_ragdoll(&mut self, force: Vec2) -> ParticleSystem {

        // Create Skeleton Ragdoll
        let mut ragdoll = ParticleSystem::from(&self.skeleton, 2);

        // Setup additional constraints for nicer looks
        // TODO move these into skeleton data and use angular constraints
        // instead
        let additional_constraint_pairs = vec![
            // Back legs
            (1, 9, 1.0),
            (1, 11, 1.0),

            // Head legs
            (3, 9, 1.00),
            (3, 11, 1.00),

            // Hip arms
            (8, 4, 1.00),
            (8, 6, 1.00)

        ];

        for (a, b, s) in additional_constraint_pairs {
            let ap = self.skeleton.get_bone_index(a).end_local();
            let bp = self.skeleton.get_bone_index(b).end_local();
            let d = (ap - bp).len() * s;
            ragdoll.add_constraint(StickConstraint::new(a, b, d));
        }

        // Tweak inverse masses of root, back and head
        ragdoll.get_mut(0).set_invmass(0.97);
        ragdoll.get_mut(1).set_invmass(0.98);
        ragdoll.get_mut(3).set_invmass(0.99);

        // Apply initial force
        ragdoll.get_mut(0).apply_force(force);
        ragdoll.get_mut(3).apply_force(force * 0.8);
        ragdoll

    }


    // Internal ---------------------------------------------------------------
    fn update_skeleton(&mut self, dt: f32, level: &Level) {

        let facing = self.skeleton.local_transform();

        // Aim Leanback
        let aim_horizon = self.compute_view_horizon_distance();
        let leanback = (
                aim_horizon * 0.5
                - self.recoil * self.config.recoil_leanback_factor

            ).min(self.config.leanback_max).max(self.config.leanback_min) * 0.009;;

        self.skeleton.get_bone_mut("Back").unwrap().set_user_angle(leanback + self.state.velocity.x * 0.05 * facing.x);
        self.skeleton.get_bone_mut("Neck").unwrap().set_user_angle(leanback * self.config.leanback_head_factor);

        // Place and update bones
        if !self.state.is_grounded {
            self.skeleton.set_animation(&JUMP_ANIMATION, (0.3 * self.state.velocity.x.abs().max(1.0).min(1.125)), 0.05);

        } else if self.state.velocity.x.abs() > 0.5 {
            if self.state.velocity.x.signum() == facing.x {
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
            self.state.position + self.config.offset + idle_offset + crouch_offset + run_offset + compression
        );

        // Animate and Arrange
        self.skeleton.animate(dt);
        self.skeleton.arrange();

        // Weapon Grip IK
        let shoulder = self.skeleton.get_bone_end_ik("Back");
        let grip_angle = Angle::transform(self.state.direction, facing);
        let grip = shoulder + Angle::offset(grip_angle, 17.0 - self.recoil) + Angle::offset(grip_angle + D90, 1.0);
        let trigger = shoulder + Angle::offset(grip_angle, 6.5 - self.recoil * 0.5) + Angle::offset(grip_angle + D90, 4.0);
        self.skeleton.apply_ik("L.Hand", grip, true);
        self.skeleton.apply_ik("R.Hand", trigger, true);

        // Leg IK
        if self.state.is_grounded {

            let mut foot_l = self.skeleton.get_bone_end_ik("L.Foot");
            let mut foot_r = self.skeleton.get_bone_end_ik("R.Foot");

            if self.collide_ground(level, &mut foot_l) {
                self.skeleton.apply_ik("L.Foot", foot_l, false);
            }
            if self.collide_ground(level, &mut foot_r) {
                self.skeleton.apply_ik("R.Foot", foot_r, false);
            }

        }

    }

    fn compute_view_horizon_distance(&self) -> f32 {
        let shoulder = self.skeleton.get_bone_end_local("Back");
        let aim = shoulder + Angle::offset(self.state.direction, self.config.line_of_sight_length);
        aim.y - shoulder.y
    }

    fn collide_ground(&mut self, level: &Level, p: &mut Vec2) -> bool {
        let floor = self.skeleton.to_local(Vec2::new(0.0, level.floor));
        if p.y > floor.y {
            p.y = floor.y;
            true

        } else {
            false
        }
    }

}

