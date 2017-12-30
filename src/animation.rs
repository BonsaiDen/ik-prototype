// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// STD Dependencies -----------------------------------------------------------
use std::collections::HashMap;


// Types ----------------------------------------------------------------------
pub type AnimationFrameBone = (&'static str, f32);
type AnimationFrame = (f32, Vec<AnimationFrameBone>);


// Animation Data Abstraction -------------------------------------------------
#[derive(Debug)]
pub struct AnimationData {
    pub duration: f32,
    pub key_frames: Vec<AnimationFrame>
}


// Animator State Machine Abstraction -----------------------------------------
#[derive(Debug, Default)]
pub struct AnimatorBuilder {
    default_blend: f32,
    blends: HashMap<(&'static str, &'static str), f32>,
    states: HashMap<&'static str, AnimatorState>
}

impl AnimatorBuilder {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_state<C: Fn(&mut AnimatorState)>(mut self, name: &'static str, callback: C) -> Self {
        let mut state = AnimatorState::new();
        callback(&mut state);
        self.states.insert(name, state);
        self
    }

    pub fn with_default_blend(mut self, duration: f32) -> Self {
        self.default_blend = duration;
        self
    }

    pub fn with_blend(mut self, from: &'static str, to: &'static str, duration: f32) -> Self {
        self.blends.insert((from, to), duration);
        self
    }

    pub fn build(self) -> Animator {
        Animator {
            default_blend: self.default_blend,
            blends: self.blends,
            speeds: HashMap::new(),
            blend_duration: 0.0,
            blend_timer: 0.0,
            states: self.states,
            previous: None,
            current: None
        }
    }

}

#[derive(Debug)]
pub struct AnimatorState {
    animations: Vec<AnimationInstance>
}

impl AnimatorState {

    fn new() -> Self {
        Self {
            animations: Vec::new()
        }
    }

    pub fn add_animation(&mut self, data: &'static AnimationData) {
        self.animations.push(AnimationInstance::new(data, 1.0));
    }

    fn update(&mut self, dt: f32, speed: f32) {
        for animation in &mut self.animations {
            animation.speed = speed;
            animation.update(dt);
        }
    }

    fn reset(&mut self) {
        for animation in &mut self.animations {
            animation.reset();
        }
    }

    fn apply_to_bones(&mut self, factor: f32, bones: &mut [AnimationFrameBone]) {
        for animation in &mut self.animations {
            // TODO merge multiple internal animations?
            animation.apply_to(bones, factor);
        }
    }

}

pub struct Animator {
    default_blend: f32,
    blends: HashMap<(&'static str, &'static str), f32>,
    speeds: HashMap<&'static str, f32>,
    states: HashMap<&'static str, AnimatorState>,
    blend_duration: f32,
    blend_timer: f32,
    previous: Option<&'static str>,
    current: Option<&'static str>
}

impl Animator {

    pub fn set_speed(&mut self, state: &'static str, factor: f32) {
        if self.speeds.contains_key(state) {
            if let Some(s) = self.speeds.get_mut(state) {
                *s = factor;
            }

        } else {
            self.speeds.insert(state, factor);
        }
    }

    pub fn transition_to(&mut self, state: &'static str) {

        // Do nothing if already in the requested state
        if let Some(current) = self.current {
            if current == state {
                return;
            }
        }

        // Do nothing if the state does not exists
        if let Some(ref mut state) = self.states.get_mut(state) {
            state.reset();

        } else {
            return;
        }

        // Blend to next state
        self.previous = self.current.take();
        self.current = Some(state);

        self.blend_duration = if let Some(previous) = self.previous {
            self.blends.get(&(previous, state)).cloned().unwrap_or_else(|| {
                self.blends.get(&("*", state)).cloned().unwrap_or_else(|| {
                    self.blends.get(&(previous, "*")).cloned().unwrap_or(self.default_blend)
                })
            })

        } else {
            self.default_blend
        };

        self.blend_timer = 0.0;

    }

    pub fn update(&mut self, dt: f32, bones: &mut [AnimationFrameBone]) {

        self.blend_timer = (self.blend_timer + dt).min(self.blend_duration);

        let blend_factor = cubic_bezier(0.0, 0.0, 1.0, 1.0, (1.0 / self.blend_duration) * self.blend_timer);
        if let Some(previous) = self.previous {
            let speed = self.speeds.get(previous).cloned().unwrap_or(1.0);
            if let Some(ref mut state) = self.states.get_mut(previous) {
                if 1.0 - blend_factor > 0.0 {
                    state.update(dt, speed);
                    state.apply_to_bones(1.0 - blend_factor, bones);
                }
            }
        }

        if let Some(current) = self.current {
            let speed = self.speeds.get(current).cloned().unwrap_or(1.0);
            if let Some(ref mut state) = self.states.get_mut(current) {
                state.update(dt, speed);
                state.apply_to_bones(blend_factor, bones);
            }
        }

    }

}


// Animation Abstraction ------------------------------------------------------
#[derive(Debug)]
pub struct AnimationInstance {
    time: f32,
    blend: f32,
    speed: f32,
    key_index: usize,
    data: &'static AnimationData
}

impl AnimationInstance {

    fn new(data: &'static AnimationData, speed: f32) -> Self {
        Self {
            time: 0.0,
            blend: 0.0,
            speed: speed,
            key_index: 0,
            data: data
        }
    }

    fn update(&mut self, dt: f32) {

        let duration = self.data.duration;
        let key_count = self.data.key_frames.len();
        let next_offset = self.data.key_frames[(self.key_index + 1) % key_count].0;

        if self.speed > 0.0 {
            self.time += dt * self.speed;
        }

        // Loop
        if next_offset == 0.0 && self.time >= duration {
            self.time -= duration;
            self.key_index = 0;

        // Advance
        } else if next_offset > 0.0 && self.time >= next_offset {
            self.key_index = (self.key_index + 1) % key_count;
        }

        // Fetch the newly updated offsets
        let prev_offset = self.data.key_frames[self.key_index].0;
        let next_offset = self.data.key_frames[(self.key_index + 1) % key_count].0;

        // blend factor between the prev and next frame
        // TODO support non-looping by not using a modulo here???
        let delta = ((next_offset - prev_offset) + duration) % duration;
        if delta == 0.0 {
            self.blend = 1.0;

        } else {
            let into = ((self.time - prev_offset) + duration) % duration;
            self.blend = 1.0 / delta * into;
        }

    }

    fn reset(&mut self) {
        self.time = 0.0;
        self.blend = 0.0;
        self.speed = 0.0;
        self.key_index = 0;
    }

    fn apply_to(&self, bones: &mut [AnimationFrameBone], factor: f32) {
        let values = self.blend();
        for  b in bones.iter_mut() {
            for v in &values[..] {
                if v.0 == b.0 {
                    b.1 += v.1 * factor;
                    break;
                }
            }
        }
    }

    fn blend(&self) -> Vec<AnimationFrameBone> {

        let key_count = self.data.key_frames.len();
        let (_, ref prev_values) = self.data.key_frames[self.key_index];
        let (_, ref next_values) = self.data.key_frames[(self.key_index + 1) % key_count];

        let mut blended_values = prev_values.clone();
        for p in &mut blended_values {
            for n in &next_values[..] {
                if n.0 == p.0 {
                    p.1 = cubic_bezier(p.1, p.1, n.1, n.1, self.blend);
                    break;
                }
            }
        }

        blended_values

    }

}

// Helpers --------------------------------------------------------------------
fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32  {
    p1 + 0.5 * t *(p2 - p0 + t * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3 + t * (3.0 * (p1 - p2) + p3 - p0)))
}

