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
    ParticleConstraint, ParticleSystem, ParticleTemplate,
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
    headband: ParticleSystem

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
            headband: ParticleTemplate::schal(1, 4, 3.5)
        }
    }

    pub fn reset(&mut self) {
        self.ragdoll.take();
    }

    pub fn kill(&mut self) {
        if self.ragdoll.is_none() {

            let facing = Angle::facing(self.state.direction + D90).to_vec();
            let mut particles = ParticleSystem::new(self.skeleton.len(), 2);

            self.skeleton.visit_with_parents(|bone, parent| {
                {
                    let mut p = particles.get_mut(bone.index());
                    p.set_invmass(1.0);
                    p.set_position(self.skeleton.to_world(bone.end().scale(facing)) * self.config.scale);
                }

                if let Some(parent) = parent {
                    particles.add_constraint(
                        ParticleConstraint::new(bone.index(), parent.index(), bone.length() * 0.49)
                    );
                }

            }, false);

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
                let d = (ap - bp).mag() * 0.49 * s;
                particles.add_constraint(
                    ParticleConstraint::new(a, b, d)
                );
            }

            particles.get_mut(0).apply_force(Vec2::new(-5.0, -10.5).scale(facing));
            particles.get_mut(3).apply_force(Vec2::new(-8.0, -20.5).scale(facing));

            particles.get_mut(0).set_invmass(0.97);
            particles.get_mut(1).set_invmass(0.98);
            particles.get_mut(3).set_invmass(0.99);

            self.ragdoll_timer = 0.0;
            self.ragdoll_facing = facing;
            self.ragdoll = Some(particles);

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
        let head = self.skeleton.get_bone("Head").unwrap().end().scale(facing);
        let head = self.skeleton.to_world(head) * self.config.scale;
        self.headband.get_mut(0).set_position(head);
        self.headband.step(dt, Vec2::new(-170.0 * facing.x, 30.0), |p| {

        });

    }

    pub fn draw(&mut self, context: &mut Context, level: &Level) {

        let ragdoll_timer = self.ragdoll_timer;
        if let Some(ref mut ragdoll) = self.ragdoll {
            // TODO level ground collision...
            // TODO disable ragdoll once we hit the ground
            ragdoll.step(context.dt(), Vec2::new(0.0, 120.0), |mut p| {
                if p.position.y > level.floor {
                    if ragdoll_timer > 1.0 {
                        p.set_invmass(0.5);
                    }
                    p.position.y = p.position.y.min(level.floor);
                }
            });
        }

        // Rag-dolled or not
        let facing = if let Some(ref ragdoll) = self.ragdoll {

            let inv_scale = 1.0 / self.config.scale;

            // Set bone positions from ragdoll particles
            let mut positions = Vec::new();
            self.skeleton.visit_with_parents(|bone, parent| {
                if let Some(parent) = parent {
                    let b = ragdoll.get(bone.index());
                    let p = ragdoll.get(parent.index());
                    positions.push((
                        bone.index(),
                        self.skeleton.to_local(b.position * inv_scale).scale(self.ragdoll_facing),
                        self.skeleton.to_local(p.position * inv_scale).scale(self.ragdoll_facing)
                    ));
                }

            }, false);

            // TODO a more effective way?
            for (index, b, p) in positions {
                self.skeleton.get_bone_index_mut(index).set_from_ragdoll(p, b);
            }

            // TODO take care of root bone position
            //let p = ragdoll.get(0);
            // self.skeleton.get_bone_index_mut(0).set_from_ragdoll(p, p);
            self.ragdoll_facing

        } else {
            let facing = Angle::facing(self.state.direction + D90).to_vec();
            self.update_bones(context.dt(), facing, level);
            facing
        };

        // Headband
        self.headband.visit_chain(|p, n, i| {
            if i == 0 {
                let d = (n.position - p.position) * 0.5;
                context.line_vec(p.position + d, n.position, 0x00ffff00);

            } else {
                context.line_vec(p.position, n.position, 0x00ffff00);
            }
        });

        // Draw bones
        self.skeleton.visit(|bone| {

            let line = (
                self.skeleton.to_world(bone.start().scale(facing)),
                self.skeleton.to_world(bone.end().scale(facing))
            );

            // Draw Head
            if bone.name() == "Head" {
                context.circle_vec(line.1 * self.config.scale, 4.0 * self.config.scale, 0x00d0d0d0);

            } else if bone.name() == "L.Arm" || bone.name() == "L.Hand" {
                context.line_vec(line.0 * self.config.scale, line.1 * self.config.scale, 0x00808080);

            } else if bone.name() == "L.Leg" || bone.name() == "L.Foot" {
                context.line_vec(line.0 * self.config.scale, line.1 * self.config.scale, 0x00808080);

            } else {
                //context.line_vec(line.0, line.1, COLORS[bone.index()]);
                context.line_vec(line.0 * self.config.scale, line.1 * self.config.scale, 0x00d0d0d0);
            }

        }, false);

        // Draw Weapon
        // TODO let weapon drop once rag-dolled
        // TODO use a rigid body as the weapon all the time
        if self.ragdoll.is_none() {

            let shoulder = self.skeleton.get_bone("Back").unwrap().end();
            let (_, grip_angle, trigger) = self.get_weapon_grip(shoulder, facing);

            let facing_shoulder = shoulder.scale(facing);
            let stock = self.skeleton.to_world(facing_shoulder + Angle::offset(self.state.direction, 0.5 - self.recoil));
            let barrel = self.skeleton.to_world(facing_shoulder + Angle::offset(self.state.direction, 30.0 - self.recoil));
            let barrel_mid = self.skeleton.to_world(facing_shoulder + Angle::offset(self.state.direction, 15.0 - self.recoil));
            let trigger_base = self.skeleton.to_world((trigger + Angle::offset(grip_angle - D90, 4.0)).scale(facing));
            let trigger = self.skeleton.to_world(trigger.scale(facing));
            let color = 0x00ffff00;

            context.line_vec(stock * self.config.scale, barrel * self.config.scale, color);
            context.line_vec(trigger * self.config.scale, trigger_base * self.config.scale, color);
            context.line_vec(stock * self.config.scale, trigger * self.config.scale, color);
            context.line_vec(barrel_mid * self.config.scale, trigger * self.config.scale, color);
        }

    }

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
        let s = 1.0 / self.config.scale;
        if !self.state.is_grounded {
            self.skeleton.set_animation(&JUMP_ANIMATION, (0.3 * self.state.velocity.x.abs().max(1.0).min(1.125)), 0.05);

        } else if self.state.velocity.x.abs() > 0.5 {
            //if self.is_crouching {
            //    self.skeleton.set_animation(&RUN_ANIMATION, 0.1, 0.05);

            if self.state.velocity.x.signum() == facing.x {
                self.skeleton.set_animation(&RUN_ANIMATION, 0.05 * s, 0.05);

            } else {
                self.skeleton.set_animation(&RUN_BACKWARDS_ANIMATION, 0.04 * s, 0.05);
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
        let floor = self.skeleton.to_local(Vec2::new(0.0, level.floor * (1.0 / self.config.scale)));
        if p.y > floor.y {
            p.y = floor.y;
            true

        } else {
            false
        }
    }

}

