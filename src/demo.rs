// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use lean::{SkeletalData, Vec2};
use super::Context;

use super::player::{Player, PlayerRenderable, Config};


// Statics --------------------------------------------------------------------
const D90: f32 = PI * 0.5;

lazy_static! {

    static ref SKELETON: SkeletalData = SkeletalData {
        bones: vec![
            (  "Root", ( "Root",  0.0, -D90)), // 0

            (  "Back", ( "Root", 17.0,  0.0)), // 1
            (  "Neck", ( "Back",  2.0,  0.0)), // 2
            (  "Head", ( "Neck",  4.0,  0.0)), // 3

            ( "L.Arm", ( "Back",  9.0, -D90)),  // 4
            ("L.Hand", ("L.Arm", 13.0,  0.0)), // 5
            ( "R.Arm", ( "Back",  9.0,  D90)), // 6
            ("R.Hand", ("R.Arm", 13.0,  0.0)), // 7

            (  "Hip", ( "Root",   1.0, D90 * 2.0)), // 8

            ( "L.Leg", (  "Hip", 13.0,  0.0)), // 9
            ("L.Foot", ("L.Leg", 14.0,  0.0)), // 10
            ( "R.Leg", (  "Hip", 13.0,  0.0)), // 11
            ("R.Foot", ("R.Leg", 14.0,  0.0)) // 12
        ]
    };
}

// Demo Code ------------------------------------------------------------------
pub struct Level {
    pub width: f32,
    pub floor: f32
}

impl Level {

    fn draw(&mut self, context: &mut Context) {
        context.line(0.0, self.floor + 1.0, self.width, self.floor + 1.0, 0x00c0c0c0);
    }

}

pub struct Demo {
    player: Player,
    renderable: PlayerRenderable,
    level: Level,
    input_direction: f32
}

impl Demo {

    pub fn new(width: f32, height: f32) -> Self {

        let config = Config {
            scale: 1.0,
            acceleration: 0.70,
            acceleration_max: 3.5,
            velocity_damping: 0.7,
            velocity_backwards_factor: 0.5,
            jump_force: 5.5,
            fall_speed: 0.25,
            fall_limit: 4.5,
            offset: Vec2::new(0.0, -25.0),
            shoulder_height: 25.0,
            line_of_sight_length: 80.0,

            leanback_min: -45.0,
            leanback_max: 35.0,
            leanback_head_factor: 1.45,
            recoil_leanback_factor: 2.0,
            recoil_force: 6.0,
            recoil_damping: 0.8,
            idle_compression: 1.25,
            idle_speed: 5.0,
            land_compression: 10.0,
            land_compression_factor: 0.99,
            land_speed: 11.5,
            run_compression: 1.5,
            run_speed: 16.0,
            crouching_factor: 0.5,
            crouch_compression: 3.0,
            crouch_speed: 1.0
        };

        Self {
            player: Player::new(config.clone()),
            renderable: PlayerRenderable::from_skeleton(&SKELETON, config),
            level: Level {
                width,
                floor: height * 0.75
            },
            input_direction: 0.0
        }

    }

    pub fn update(
        &mut self,
        mouse_pos: Option<(f32, f32)>,
        left: bool,
        right: bool,
        crouch: bool,
        jump: bool,
        fire: bool,
        kill: bool,
        reset: bool
    ) {

        if let Some((x, y)) = mouse_pos {
            self.input_direction = self.player.compute_view_angle(Vec2::new(x, y));
        }

        if kill {
            self.renderable.kill();
        }

        if reset {
            self.renderable.reset();
        }

        self.player.update_server(fire);
        self.player.update_shared(left, right, crouch, jump, self.input_direction, &self.level);

    }

    pub fn draw(&mut self, context: &mut Context) {
        self.renderable.set_state(self.player.get_state());
        self.renderable.update(0.0166666);
        self.renderable.draw(context, &self.level);
        self.level.draw(context);
    }

}

