// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Crates ---------------------------------------------------------------------
extern crate lean;
extern crate line_drawing;
extern crate minifb;


// STD Dependencies -----------------------------------------------------------
use std::thread;
use std::time::{self, Duration, Instant};


// External Dependencies ------------------------------------------------------
use minifb::{Key, KeyRepeat, WindowOptions, Window, Scale, MouseMode, MouseButton};
use line_drawing::{BresenhamCircle, Midpoint};
use lean::Vec2;
use lean::library::Renderer;


// Modules --------------------------------------------------------------------
mod stick_example;
use self::stick_example::Example;


// Main -----------------------------------------------------------------------
fn main() {

    let scale = 0.5;
    let inv_scale = 1.0 / scale;
    let width = (640.0 * scale) as usize;
    let height = (480.0 * scale) as usize;

    let mut window = Window::new(
        "IK Prototype - ESC to exit",
        width,
        height,
        WindowOptions {
           scale: Scale::X2,
            .. WindowOptions::default()
        }).unwrap_or_else(|e| {
            panic!("{}", e);
        });


    let mut context = Context {
        width: width,
        height: height,
        scale: scale,
        buffer: vec![0; width * height],
        dt: 0.0
    };

    let mut last_wait = Instant::now();
    let mut accumulated_wait = Duration::from_millis(0);
    let mut last_frame = precise_time_ms();

    let mut example = Example::new(width as f32 * inv_scale, height as f32 * inv_scale);
    while window.is_open() && !window.is_key_down(Key::C) {

        for i in &mut context.buffer {
            *i = 0;
        }

        let mouse_pos = if let Some((x, y)) = window.get_mouse_pos(MouseMode::Discard) {
            Some((x * (1.0 / context.scale), y * (1.0 / context.scale)))

        } else {
            None
        };

        example.update(
            mouse_pos,
            window.is_key_down(Key::A),
            window.is_key_down(Key::D),
            false,//window.is_key_down(Key::LeftShift),
            window.is_key_down(Key::Space),
            window.get_mouse_down(MouseButton::Left),
            window.get_mouse_down(MouseButton::Right),
            window.is_key_down(Key::Enter),
            window.is_key_down(Key::R),
            window.is_key_down(Key::P),
            window.is_key_pressed(Key::B, KeyRepeat::No)
        );

        let t = precise_time_ms();
        let d = (t - last_frame) as f32 / 1000.0;
        last_frame = t;
        context.dt = d as f32;
        example.draw(&mut context);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&context.buffer).unwrap();

        // Calculate desired wait time
        let desired_wait = Duration::new(0, 1_000_000_000 / 60 as u32);

        // Calculate additional time taken by external logic
        accumulated_wait += last_wait.elapsed();

        // If the accumulated wait is lower than the desired_wait wait, simply subtract it
        if accumulated_wait <= desired_wait {
            thread::sleep(desired_wait - accumulated_wait);
            accumulated_wait = Duration::new(0, 0);

        // Otherwise reduce the accumulated wait by desired_wait and do not sleep at all
        } else {
            accumulated_wait -= desired_wait;
        }

        last_wait = Instant::now();

    }

}

pub struct Context {
    width: usize,
    height: usize,
    buffer: Vec<u32>,
    scale: f32,
    dt: f32
}

impl Context {

    pub fn dt(&self)-> f32 {
        self.dt
    }

    pub fn circle_vec(&mut self, c: Vec2, r: f32, color: u32) {
        self.circle(c.x, c.y, r, color);
    }

    pub fn circle(&mut self, x: f32, y: f32, r: f32, color: u32) {
        for (x, y) in BresenhamCircle::new((x * self.scale) as i32, (y * self.scale) as i32, (r * self.scale) as i32) {
            if x > 0 && x < self.width as i32 && y > 0 && y < self.height as i32 {
                self.buffer[y as usize * self.width + x as usize] = color;
            }
        }
    }

    pub fn line_vec(&mut self, start: Vec2, end: Vec2, color: u32) {
        self.line(start.x, start.y, end.x, end.y, color);
    }

    pub fn line(&mut self, sx: f32, sy: f32, tx: f32, ty: f32, color: u32) {
        /*
        for ((x, y), value) in XiaolinWu::<f32, i32>::new((sx, sy), (tx, ty)) {

            let r = (((color & 0x00ff0000) >> 16) as f32 * value) as u32;
            let g = (((color & 0x0000ff00) >> 8) as f32 * value) as u32;
            let b = (((color & 0x000000ff)) as f32 * value) as u32;

            let c = b | (g << 8) | r << 16;

            if x > 0 && x < self.width as i32 && y > 0 && y < self.height as i32 {
                self.buffer[y as usize * self.width + x as usize] = c;
            }
        }*/
        for (x, y) in Midpoint::<f32, i32>::new((sx * self.scale, sy * self.scale), (tx * self.scale, ty * self.scale)) {
            if x > 0 && x < self.width as i32 && y > 0 && y < self.height as i32 {
                self.buffer[y as usize * self.width + x as usize] = color;
            }
        }
    }

}

impl Renderer for Context {
    fn dt(&self)-> f32 {
        self.dt
    }

    fn draw_circle(&mut self, c: Vec2, r: f32, color: u32) {
        self.circle(c.x, c.y, r, color);
    }

    fn draw_line(&mut self, start: Vec2, end: Vec2, color: u32) {
        self.line(start.x, start.y, end.x, end.y, color);
    }

    fn draw_rect(&mut self, tr: Vec2, bl: Vec2, color: u32) {

        let size = bl - tr;

        // Top
        self.line(tr.x, tr.y, tr.x + size.x, tr.y, color);

        // Bottom
        self.line(bl.x - size.x, bl.y, bl.x, bl.y, color);

        // Right
        self.line(tr.x, tr.y, tr.x, tr.y + size.y, color);

        // Left
        self.line(bl.x, bl.y - size.y, bl.x, bl.y, color);

    }

}


fn precise_time_ms() -> u64 {

    let dur = match time::SystemTime::now().duration_since(time::UNIX_EPOCH) {
        Ok(dur) => dur,
        Err(err) => err.duration(),
    };

    dur.as_secs() * 1000 + u64::from(dur.subsec_nanos() / 1_000_000)

}


