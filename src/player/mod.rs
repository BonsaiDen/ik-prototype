// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::f32::consts::PI;


// Internal Dependencies ------------------------------------------------------
use lean::{Angle, Vec2, f32_equals};
use lean::library::{StickFigureConfig, StickFigureState};
use ::demo::Level;


#[derive(Clone)]
pub struct PlayerState {
    hp: u8,
    position: Vec2,
    velocity: Vec2,
    direction: f32,
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

impl StickFigureState for PlayerState {

    fn is_alive(&self) -> bool {
        self.hp > 0
    }

    fn position(&self) -> Vec2 {
        self.position
    }

    fn velocity(&self) -> Vec2 {
        self.velocity
    }

    fn direction(&self) -> f32 {
        self.direction
    }

    fn is_grounded(&self) -> bool {
        self.is_grounded
    }

    fn is_crouching(&self) -> bool {
        self.is_crouching
    }

    fn is_firing(&self) -> bool {
        self.is_firing
    }

}

pub struct Player {

    // Server only
    ticks_since_firing: usize,

    // Shared with Server
    state: PlayerState,
    config: StickFigureConfig

}

impl Player {

    pub fn new(config: StickFigureConfig) -> Self {
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
        let vel_factor = if f32_equals(self.state.velocity.x.signum(), facing.x) || !self.state.is_grounded { 1.0 } else { self.config.velocity_backwards_factor  };
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

