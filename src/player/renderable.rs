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
    ParticleConstraint, ParticleSystem, ParticleTemplate, RigidBodyData, RigidBody
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
            ("StockHigh", 0.0, 0.0),
            ("StockLow", 0.0, 5.0),
            ("UpperMid", 0.0, -15.0)
        ],
        constraints: vec![
            ("Center", "Barrel", true),
            ("Center", "StockHigh", true),
            ("Center", "StockLow", true),
            ("StockHigh", "StockLow", true),
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
    headband: ParticleSystem,
    weapon: RigidBody

}

impl PlayerRenderable {

    pub fn from_skeleton(data: &'static SkeletalData, config: Config) -> Self {
        Self {

            config: config,
            state: PlayerState::new(),

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
            headband: ParticleTemplate::schal(1, 4, 7.0),
            weapon: RigidBody::new(&WEAPON_RIGID)
        }
    }

    pub fn set_state(&mut self, state: PlayerState) {
        self.state = state;
    }

    pub fn update(&mut self, dt: f32) {

        // Rag-dolled or not
        let facing = if self.ragdoll.is_some() {
            self.ragdoll_timer += dt;
            self.ragdoll_facing

        } else {
            self.update_active(dt);
            Angle::facing(self.state.direction + D90).to_vec()
        };

        // Update headband
        let neck = self.skeleton.get_bone("Neck").unwrap().end().scale(facing);
        let neck = self.skeleton.to_world(neck);
        self.headband.get_mut(0).set_position(neck);

    }

    pub fn draw(&mut self, context: &mut Context, level: &Level) {

        // Update ragdoll
        let ragdoll_timer = self.ragdoll_timer;
        if let Some(ref mut ragdoll) = self.ragdoll {
            ragdoll.step(context.dt(), Vec2::new(0.0, 240.0), |mut p| {
                if p.position.y > level.floor {
                    if ragdoll_timer > 1.0 {
                        p.set_invmass(0.5);
                    }
                    p.position.y = p.position.y.min(level.floor);
                }
            });
        }

        // Ragdoll or skeleton driven drawing
        let facing = if let Some(ref ragdoll) = self.ragdoll {

            // Set bone positions from ragdoll particles
            let mut positions = Vec::new();
            self.skeleton.visit_with_parents(|bone, parent| {
                if let Some(parent) = parent {
                    let b = ragdoll.get(bone.index());
                    let p = ragdoll.get(parent.index());
                    positions.push((
                        bone.index(),
                        self.skeleton.to_local(b.position).scale(self.ragdoll_facing),
                        self.skeleton.to_local(p.position).scale(self.ragdoll_facing)
                    ));
                }

            }, false);

            // TODO a more effective way?
            for (index, b, p) in positions {
                self.skeleton.get_bone_index_mut(index).set_from_ragdoll(p, b);
            }

            self.ragdoll_facing

        } else {
            let facing = Angle::facing(self.state.direction + D90).to_vec();
            self.update_bones(context.dt(), facing, level);
            facing
        };

        // Headband
        self.headband.activate(); // Don't let the headband fall into sleep
        // TODO make the headband flail up and down in the wind using a sin() on some global timer
        self.headband.step(context.dt(), Vec2::new(-200.0 * facing.x, 60.0), |p| {
            p.position.y = p.position.y.min(level.floor);
        });

        self.headband.visit_particles_chained(|_, p, n, _| {
            context.line_vec(p.position, n.position, 0x00ffff00);
        });

        // Draw bones
        self.skeleton.visit(|bone| {

            let line = (
                self.skeleton.to_world(bone.start().scale(facing)),
                self.skeleton.to_world(bone.end().scale(facing))
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

            let shoulder = self.skeleton.get_bone("Back").unwrap().end();
            let facing_shoulder = shoulder.scale(facing);

            self.weapon.update(
                self.skeleton.to_world(facing_shoulder),
                Vec2::new(-self.recoil, 0.0),
                facing.flipped(),
                self.state.direction
            );

            self.weapon.visit_static(|a, b| {
                context.line_vec(
                    a,
                    b,
                    0x00ff0000
                );
            });

        } else {
            self.weapon.step(context.dt(), Vec2::new(0.0, 240.0), |p| {
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
                    0x00ff0000
                );
            });
        }

    }

    pub fn reset(&mut self) {
        self.ragdoll.take();
    }

    pub fn kill(&mut self) {
        if self.ragdoll.is_none() {

            let facing = Angle::facing(self.state.direction + D90).to_vec();
            let force = Vec2::new(-16.0, -31.0).scale(facing);

            // Update weapon model to support ragdoll
            self.weapon.update_ragdoll();
            self.weapon.apply_force(force * 0.5);

            // Create Skeleton Ragdoll
            let mut particles = ParticleSystem::new(self.skeleton.len(), 2);

            self.skeleton.visit_with_parents(|bone, parent| {
                {
                    let p = particles.get_mut(bone.index());
                    p.set_invmass(1.0);
                    p.set_position(self.skeleton.to_world(bone.end().scale(facing)));
                }

                if let Some(parent) = parent {
                    particles.add_constraint(
                        ParticleConstraint::new(bone.index(), parent.index(), bone.length())
                    );
                }

            }, false);

            // Setup additional constraints for nicer looks
            let constraint_pairs = vec![
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

            for (a, b, s) in constraint_pairs {
                let ap = self.skeleton.get_bone_index(a).end();
                let bp = self.skeleton.get_bone_index(b).end();
                let d = (ap - bp).mag() * s;
                particles.add_constraint(
                    ParticleConstraint::new(a, b, d)
                );
            }

            // Tweak inverse masses of root, back and head
            particles.get_mut(0).set_invmass(0.97);
            particles.get_mut(1).set_invmass(0.98);
            particles.get_mut(3).set_invmass(0.99);

            // Apply initial force
            particles.get_mut(0).apply_force(force);
            particles.get_mut(3).apply_force(force * 0.8);

            self.ragdoll_timer = 0.0;
            self.ragdoll_facing = facing;
            self.ragdoll = Some(particles);

        }
    }


    // Internal ---------------------------------------------------------------
    fn update_active(&mut self, dt: f32) {

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

    fn update_bones(&mut self, dt: f32, facing: Vec2, level: &Level) {

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

        // TODO walk compression
        self.skeleton.set_world_offset(
            self.state.position + self.config.offset + idle_offset + crouch_offset + run_offset + compression
        );

        // Animate and Arrange
        self.skeleton.animate(dt);
        self.skeleton.arrange();

        // Weapon Grip IK
        let shoulder = self.skeleton.get_bone("Back").unwrap().end();
        let (grip, _, trigger) = self.get_weapon_grip(shoulder, facing);
        self.skeleton.apply_ik("L.Hand", grip, true);
        self.skeleton.apply_ik("R.Hand", trigger, true);

        // Leg IK
        if self.state.is_grounded {

            let mut foot_l = self.skeleton.get_bone("L.Foot").unwrap().end();
            let mut foot_r = self.skeleton.get_bone("R.Foot").unwrap().end();

            if self.collide_ground(level, &mut foot_l) {
                self.skeleton.apply_ik("L.Foot", foot_l, false);
            }
            if self.collide_ground(level, &mut foot_r) {
                self.skeleton.apply_ik("R.Foot", foot_r, false);
            }

        }

    }

    fn get_weapon_grip(&self, shoulder: Vec2, facing: Vec2) -> (Vec2, f32, Vec2) {
        let grip_angle = Angle::transform(self.state.direction, facing);
        let grip = shoulder + Angle::offset(grip_angle, 17.0 - self.recoil) + Angle::offset(grip_angle + D90, 1.0);
        let trigger = shoulder + Angle::offset(grip_angle, 6.5 - self.recoil * 0.5) + Angle::offset(grip_angle + D90, 4.0);
        (grip, grip_angle, trigger)
    }

    fn compute_view_horizon_distance(&self) -> f32 {
        let shoulder = self.skeleton.get_bone("Back").unwrap().end();
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

