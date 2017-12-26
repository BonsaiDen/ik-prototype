// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use ::{Angle, Vec2, Skeleton, RigidBody, RigidBodyData};
use ::library::{Accessory, Renderer, Collider};


// Statics --------------------------------------------------------------------
lazy_static! {

    static ref WEAPON_RIFLE_RIGID: RigidBodyData = RigidBodyData {
        points: vec![
            ("Center", 14.0, 0.0),
            ("Barrel", 30.0, 0.0),
            ("StockMid", 0.0, 0.0),
            ("StockLow", 0.0, 7.0),
        ],
        constraints: vec![
            ("Center", "Barrel", true),
            ("Center", "StockMid", true),
            ("Center", "StockLow", true),
            ("StockMid", "StockLow", true),
            ("StockLow", "Barrel", false)
        ],
        iks: vec![
            ("L.Hand", 17.0, 1.0, true),
            ("R.Hand", 9.0, 4.0, true)
        ]
    };

}


// Generic Weapon Abstraction -------------------------------------------------
pub struct Weapon {
    bone: &'static str,
    color: u32,
    has_ragdoll: bool,
    ragdoll_timer: f32,
    gravity: Vec2,
    direction: f32,
    recoil: f32,
    rigid: RigidBody
}

impl Weapon {

    pub fn default(color: u32) -> Self {
        Weapon::new(color, &WEAPON_RIFLE_RIGID)
    }

    pub fn new(color: u32, model: &'static RigidBodyData) -> Self {
        Self {
            bone: "Root",
            color: color,
            has_ragdoll: false,
            ragdoll_timer: 0.0,
            gravity: Vec2::zero(),
            direction: 0.0,
            recoil: 0.0,
            rigid: RigidBody::new(model)
        }
    }

    pub fn set_recoil(&mut self, recoil: f32) {
        self.recoil = recoil;
    }

    pub fn set_aim_direction(&mut self, direction: f32) {
        self.direction = direction;
    }

}

impl<R: Renderer, C: Collider> Accessory<R, C> for Weapon {

    fn set_bone(&mut self, bone: &'static str) {
        self.bone = bone;
    }

    fn attach(&mut self, _: &Skeleton) {
        self.ragdoll_timer = 0.0;
        self.has_ragdoll = false;
    }

    fn attached(&self) -> bool {
        !self.has_ragdoll
    }

    fn detach(&mut self, _: &Skeleton) {
        if !self.has_ragdoll {
            self.has_ragdoll = true;
            self.rigid.make_dynamic();
        }
    }

    fn apply_force(&mut self, force: Vec2) {
        self.ragdoll_timer = 0.0;
        self.rigid.apply_dynamic_force(force);
    }

    fn get_iks(&self, skeleton: &Skeleton) -> Option<Vec<(&'static str, Vec2, bool)>> {
        if self.has_ragdoll {
            None

        } else {
            let shoulder = skeleton.get_bone_end_ik(self.bone);
            Some(self.rigid.iks_static(shoulder))
        }
    }

    fn fixate(&mut self, skeleton: &Skeleton) {
        if !self.has_ragdoll {

            let shoulder = skeleton.get_bone_end_world(self.bone);
            let facing = Angle::facing(self.direction + PI * 0.5).to_vec();

            self.rigid.step_static(
                shoulder,
                Vec2::new(-self.recoil, 0.0),
                facing.flipped(),
                self.direction
            );

        }
    }

    fn set_gravity(&mut self, gravity: Vec2) {
        self.gravity = gravity;
    }

    fn step(&mut self, dt: f32, collider: &C) {
        if self.has_ragdoll {
            self.ragdoll_timer += dt;

            let ragdoll_timer = self.ragdoll_timer;
            self.rigid.step_dynamic(dt, self.gravity, |p| {
                if collider.world(&mut p.position) {
                    if ragdoll_timer > 1.0 {
                        p.set_invmass(0.5);
                    }
                }
            });
        }
    }

    fn draw(&self, renderer: &mut R) {
        if self.has_ragdoll {
            self.rigid.visit_dynamic(|(_, a), (_, b), visible| {
                if visible {
                    renderer.draw_line(
                        a,
                        b,
                        self.color
                    );
                }
            });

        } else {
            self.rigid.visit_static(|a, b| {
                renderer.draw_line(
                    a,
                    b,
                    self.color
                );
            });
        }
    }

}

