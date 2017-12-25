// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use lean::{Angle, Vec2, Skeleton, RigidBody, RigidBodyData};
use lean::library::{Attachement, Renderer, Collider};


// Statics --------------------------------------------------------------------
lazy_static! {

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

// Standard Rifle Rigid Body --------------------------------------------------
pub struct StandardRifle {
    has_ragdoll: bool,
    ragdoll_timer: f32,
    gravity: Vec2,
    rigid: RigidBody
}

impl StandardRifle {

    pub fn new() -> Self {
        Self {
            has_ragdoll: false,
            ragdoll_timer: 0.0,
            gravity: Vec2::zero(),
            rigid: RigidBody::new(&WEAPON_RIGID)
        }
    }

}

impl<R: Renderer, C: Collider> Attachement<R, C> for StandardRifle {

    fn loosen(&mut self, _: &Skeleton) {
        self.has_ragdoll = true;
        self.rigid.make_dynamic();
    }

    fn fasten(&mut self, _: &Skeleton) {
        self.ragdoll_timer = 0.0;
        self.has_ragdoll = false;
    }

    fn apply_force(&mut self, force: Vec2) {
        self.rigid.apply_dynamic_force(force);
    }

    // TODO Figure out how to cleanly allow access to custom figure properties
    fn get_iks(&self, skeleton: &Skeleton, direction: f32, custom_offset: f32) -> Option<Vec<(&'static str, Vec2, bool)>> {
        if self.has_ragdoll {
            None

        } else {
            // TODO set attachment bone from the outside
            let shoulder = skeleton.get_bone_end_ik("Back");
            let facing = Angle::facing(direction + PI * 0.5).to_vec();

            let grip_angle = Angle::transform(direction, facing);
            let grip = shoulder + Angle::offset(grip_angle, 17.0 + custom_offset) + Angle::offset(grip_angle + PI * 0.5, 1.0);
            let trigger = shoulder + Angle::offset(grip_angle, 6.5 + custom_offset * 0.5) + Angle::offset(grip_angle + PI * 0.5, 4.0);

            Some(vec![
                 ("L.Hand", grip, true),
                ("R.Hand", trigger, true)
            ])
        }
    }

    // TODO Figure out how to cleanly allow access to custom figure properties
    fn fixate(&mut self, skeleton: &Skeleton, direction: f32, custom_offset: f32) {
        if !self.has_ragdoll {

            // TODO set attachment bone from the outside
            let shoulder = skeleton.get_bone_end_world("Back");
            let facing = Angle::facing(direction + PI * 0.5).to_vec();

            self.rigid.step_static(
                shoulder,
                Vec2::new(custom_offset, 0.0),
                facing.flipped(),
                direction
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
            self.rigid.visit_dynamic(|(_, a), (_, b), _| {
                renderer.draw_line(
                    a,
                    b,
                    0x00ff_ff00
                );
            });

        } else {
            self.rigid.visit_static(|a, b| {
                renderer.draw_line(
                    a,
                    b,
                    0x00ff_ff00
                );
            });
        }
    }

}

