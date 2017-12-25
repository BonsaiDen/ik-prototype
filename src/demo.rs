// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use lean::Vec2;
use lean::library::{StickFigure, StickFigureConfig};

use super::Context;
use super::player::{Player, PlayerState};


// Demo Code ------------------------------------------------------------------
pub struct Level {
    pub width: f32,
    pub floor: f32
}

impl Level {

    fn draw(&mut self, context: &mut Context) {
        context.line(0.0, self.floor + 1.0, self.width, self.floor + 1.0, 0x00c0_c0c0);
    }

}

pub struct Demo {
    player: Player,
    figure: StickFigure<PlayerState>,
    level: Level,
    input_direction: f32
}

impl Demo {

    pub fn new(width: f32, height: f32) -> Self {

        let config = StickFigureConfig {
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

        let player = Player::new(config.clone());
        let figure = StickFigure::default(player.get_state(), config);
        Self {
            player: player,
            figure: figure,
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
            self.player.set_hp(0);
        }

        if reset {
            self.player.set_hp(255);
        }

        self.player.update_server(fire);
        self.player.update_shared(left, right, crouch, jump, self.input_direction, &self.level);

    }

    pub fn draw(&mut self, context: &mut Context) {

        self.figure.set_state(self.player.get_state());

        let floor_world = Vec2::new(0.0, self.level.floor);
        let floor_local = self.figure.to_local(floor_world);

        self.figure.draw(context, |p| {
            if p.y > floor_local.y {
                p.y = p.y.min(floor_local.y);
                true

            } else {
                false
            }

        }, |p| {
            if p.y > floor_world.y {
                p.y = p.y.min(floor_world.y);
                true

            } else {
                false
            }
        });
        self.level.draw(context);
    }

}

