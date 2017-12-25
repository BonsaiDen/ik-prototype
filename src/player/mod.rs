// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use lean::{Angle, Vec2};
use super::Context;
use ::demo::Level;

mod renderable;
pub use self::renderable::PlayerRenderable;

#[derive(Clone)]
pub struct Config {
    pub acceleration: f32,
    pub acceleration_max: f32,
    pub velocity_damping: f32,
    pub velocity_backwards_factor: f32,
    pub jump_force: f32,
    pub fall_speed: f32,
    pub fall_limit: f32,
    pub offset: Vec2,
    pub shoulder_height: f32,
    pub line_of_sight_length: f32,

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

#[derive(Clone)]
pub struct PlayerState {
    position: Vec2,
    velocity: Vec2,
    direction: f32,
    hp: u8,
    is_crouching: bool,
    is_firing: bool,
    is_grounded: bool
}

impl PlayerState {
    fn new() -> Self {
        Self {
            position: Vec2::new(160.0, 120.0),
            velocity: Vec2::zero(),
            direction: 0.0,
            hp: 255,
            is_crouching: false,
            is_firing: false,
            is_grounded: false
        }
    }
}

pub struct Player {

    // Server only
    ticks_since_firing: usize,

    // Shared with Server
    state: PlayerState,
    config: Config

}

impl Player {

    pub fn new(config: Config) -> Self {
        Self {
            ticks_since_firing: 0,

            config: config,
            state: PlayerState::new(),
        }
    }

    pub fn set_hp(&mut self, hp: u8) {
        self.state.hp = hp;
    }

    pub fn get_state(&self) -> PlayerState {
        self.state.clone()
    }

    pub fn update_server(&mut self, fire: bool) {
        if fire && self.ticks_since_firing == 0 {
            self.state.is_firing = true;
            self.ticks_since_firing = 10;

        } else {
            self.state.is_firing = false;
            self.ticks_since_firing = self.ticks_since_firing.saturating_sub(1);
        }
    }

    pub fn update_shared(
        &mut self,
        left: bool,
        right: bool,
        crouch: bool,
        jump: bool,
        direction: f32,
        level: &Level
    ) {

        // Movement
        if left {
            self.state.velocity.x -= self.config.acceleration;
        }

        if right {
            self.state.velocity.x += self.config.acceleration;
        }

        if !left && !right {
            self.state.velocity.x *= self.config.velocity_damping;
        }

        self.state.velocity.x = self.state.velocity.x.max(-self.config.acceleration_max).min(self.config.acceleration_max);
        if self.state.velocity.x < 0.01 && self.state.velocity.x > -0.01 {
            self.state.velocity.x = 0.0;
        }

        // Jumping
        if jump && self.state.is_grounded {
            self.state.is_grounded = false;
            self.state.velocity.y -= self.config.jump_force;
        }

        // Crouching
        if self.state.is_grounded && crouch {
            self.state.is_crouching = true;

        } else {
            self.state.is_crouching = false;
        }

        // Physics
        self.state.velocity.y = (self.state.velocity.y + self.config.fall_speed).min(self.config.fall_limit);

        let facing = Angle::facing(self.state.direction + PI * 0.5).to_vec();
        let vel_factor = if self.state.velocity.x.signum() == facing.x || !self.state.is_grounded { 1.0 } else { self.config.velocity_backwards_factor  };
        if self.state.is_crouching {
            self.state.position.x += self.state.velocity.x * self.config.crouching_factor * vel_factor;

        } else {
            self.state.position.x += self.state.velocity.x * vel_factor;
        }
        self.state.position.y += self.state.velocity.y;

        // Collision
        self.collide_with_level(level);

        // Aiming
        self.state.direction = Angle::interpolate(self.state.direction, direction, PI * 0.1);

    }

    pub fn compute_view_angle(&self, at: Vec2) -> f32 {
        let shoulder_height = self.config.shoulder_height;
        (at - self.state.position + Vec2::new(0.0, shoulder_height) - self.config.offset).angle()
    }

    fn collide_with_level(&mut self, level: &Level) {

        if self.state.position.x < 0.0 {
            self.state.position.x = 0.0;
            self.state.velocity.x = 0.0;
        }

        if self.state.position.x > level.width {
            self.state.position.x = level.width;
            self.state.velocity.x = 0.0;
        }

        if self.state.position.y > level.floor {
            self.state.position.y = level.floor;
            self.state.velocity.y = 0.0;
            self.state.is_grounded = true;
        }

    }

}

