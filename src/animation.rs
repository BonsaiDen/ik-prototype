// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Types ----------------------------------------------------------------------
pub type AnimationFrameBone = (&'static str, f32);
type AnimationFrame = (f32, Vec<AnimationFrameBone>);


// Animation Data Abstraction -------------------------------------------------
pub struct AnimationData {
    pub name: &'static str,
    pub duration: f32,
    pub key_frames: Vec<AnimationFrame>
}


// Animation Blender Abstraction ----------------------------------------------
pub struct AnimationBlender {
    previous: Option<Animation>,
    current: Option<Animation>,
    blend: f32,
    timer: f32,
    duration: f32
}

impl AnimationBlender {

    pub fn new() -> Self {
        Self {
            previous: None,
            current: None,
            blend: 0.0,
            timer: 0.0,
            duration: 0.0
        }
    }

    pub fn set(&mut self, animation: &'static AnimationData, blend_duration: f32, speed_factor: f32) {

        // Ignore setting the same animation twice
        if let Some(ref current) = self.current {
            if current.name() == animation.name {
                return;
            }
        }

        self.previous = self.current.take();
        self.current = Some(Animation::new(animation, speed_factor));
        self.timer = 0.0;
        self.duration = blend_duration;

    }

    pub fn update(&mut self, dt: f32, bones: &mut [AnimationFrameBone]) {

        self.timer = (self.timer + dt).min(self.duration);
        self.blend = (1.0 / self.duration) * self.timer;

        let blend = cubic_bezier(0.0, 0.0, 1.0, 1.0, self.blend);
        //println!("{:?}: {:?}", self.blend, blend);

        if let Some(ref mut previous) = self.previous {
            if 1.0 - blend > 0.0 {
                previous.update(dt);
                previous.apply_to(bones, 1.0 - blend);
            }
        }

        if let Some(ref mut current) = self.current {
            current.update(dt);
            current.apply_to(bones, blend);
        }

    }

}


// Animation Abstraction ------------------------------------------------------
pub struct Animation {
    time: f32,
    blend: f32,
    scale: f32,
    key_index: usize,
    data: &'static AnimationData
}

impl Animation {

    pub fn new(data: &'static AnimationData, scale: f32) -> Self {
        Self {
            time: 0.0,
            blend: 0.0,
            scale: scale,
            key_index: 0,
            data: data
        }
    }

    pub fn name(&self) -> &'static str {
        self.data.name
    }

    pub fn update(&mut self, dt: f32) {

        let duration = self.data.duration * self.scale;
        let key_count = self.data.key_frames.len();
        let next_offset = self.data.key_frames[(self.key_index + 1) % key_count].0 * self.scale;

        // println!("{}", dt);
        self.time += dt;

        // Loop
        if next_offset == 0.0 && self.time >= duration {
            self.time -= duration;
            self.key_index = 0;

        // Advance
        } else if next_offset > 0.0 && self.time >= next_offset {
            self.key_index = (self.key_index + 1) % key_count;
        }

        // Fetch the newly updated offsets
        let prev_offset = self.data.key_frames[self.key_index].0 * self.scale;
        let next_offset = self.data.key_frames[(self.key_index + 1) % key_count].0 * self.scale;

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

    pub fn blend(&self) -> Vec<AnimationFrameBone> {

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

    pub fn apply_to(&self, bones: &mut [AnimationFrameBone], factor: f32) {
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

}

// Helpers --------------------------------------------------------------------
fn cubic_bezier(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32  {
    p1 + 0.5 * t *(p2 - p0 + t * (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3 + t * (3.0 * (p1 - p2) + p3 - p0)))
}

