// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use super::Vec2;


// Simple Verlet based Particle System ----------------------------------------
#[derive(Default, Copy, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub prev_position: Vec2,
    m_position: Vec2,
    acceleration: Vec2,
    inv_mass: f32
}

impl Particle {

    fn new(position: Vec2) -> Self {
        Self {
            position: position,
            prev_position: position,
            m_position: position,
            acceleration: Vec2::zero(),
            inv_mass: 1.0
        }
    }

    pub fn set_invmass(&mut self, mass: f32) {
        self.inv_mass = mass;
    }

    pub fn set_position(&mut self, p: Vec2) {
        self.position = p;
        self.prev_position = p;
    }

    pub fn apply_force(&mut self, p: Vec2) {
        self.position = self.position + p;
    }

}

pub struct ParticleConstraint {
    // TODO support different type of constraints
    a: usize,
    b: usize,
    rest_length: f32
}

impl ParticleConstraint {

    pub fn new(a: usize, b: usize, rest_length: f32) -> Self {
        Self {
            a,
            b,
            rest_length
        }
    }

}

pub struct ParticleSystem {
    particles: Vec<Particle>,
    constraints: Vec<ParticleConstraint>,
    iterations: usize,
    activity: usize
}

impl ParticleSystem {

    pub fn new(max_particles: usize, iterations: usize) -> Self {

        let mut particles = Vec::with_capacity(max_particles);
        for _ in 0..max_particles {
            particles.push(Particle::new(Vec2::new(0.0, 0.0)));
        }

        Self {
            particles: particles,
            constraints: Vec::new(),
            iterations,
            activity: 10
        }

    }

    pub fn init<C: FnMut(usize, &mut Particle)>(&mut self, mut callback: C) {
        for (index, p) in self.particles.iter_mut().enumerate() {
            callback(index, p);
        }
    }

    pub fn active(&self) -> bool {
        self.activity > 0
    }

    pub fn activate(&mut self) {
        self.activity = 10;
    }

    pub fn get(&self, index: usize) -> &Particle {
        &self.particles[index]
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Particle {
        &mut self.particles[index]
    }

    pub fn add_constraint(&mut self, constraint: ParticleConstraint) {
        self.constraints.push(constraint);
        self.activate();
    }

    pub fn visit<C: FnMut(usize, &Particle, bool)>(&self, mut callback: C) {
        let is_awake = self.active();
        for (index, p) in self.particles.iter().enumerate() {
            callback(index, p, is_awake);
        }
    }

    pub fn visit_chain<C: FnMut(usize, &Particle, &Particle, bool)>(&mut self, mut callback: C) {
        let is_awake = self.active();
        for i in 1..self.particles.len() {
            callback(i - 1, &self.particles[i - 1], &self.particles[i], is_awake);
        }
    }

    pub fn step<C: Fn(&mut Particle)>(&mut self, time_step: f32, gravity: Vec2, collision: C) {
        if self.active() {
            self.accumulate_forces(gravity);
            self.verlet(time_step);
            self.satisfy_constraints(collision);
        }
    }

    fn verlet(&mut self, time_step: f32) {
        for p in &mut self.particles {
            let current_pos = p.position;
            let change = p.position - p.prev_position + p.acceleration * time_step * time_step;
            p.position = p.position + change * p.inv_mass;
            p.prev_position = current_pos;
        }
    }

    fn accumulate_forces(&mut self, gravity: Vec2) {

        // All particles are affected by gravity
        for p in &mut self.particles {
            p.acceleration = gravity;
        }

    }

    fn satisfy_constraints<C: Fn(&mut Particle)>(&mut self, collision: C) {

        let mut any_particle_active = false;
        for _ in 0..self.iterations {

            for mut p in &mut self.particles {

                collision(&mut p);

                // Check if the particle moved within the previous iteration
                if (p.position - p.m_position).len().abs() > 0.1 {
                    any_particle_active = true;
                    p.m_position = p.position;
                }

            }

            for c in &self.constraints {

                let i1 = self.particles[c.a].inv_mass;
                let i2 = self.particles[c.b].inv_mass;

                if i1 + i2 > 0.0 {

                    let p1 = self.particles[c.a].position;
                    let p2 = self.particles[c.b].position;
                    let delta = p2 - p1;

                    // Fast inverse square root
                    let dot = delta * delta;
                    let x2 = dot * 0.5;
                    let x = 0x5f375a86 - (dot.to_bits() >> 1);
                    let y = f32::from_bits(x);
                    let delta_length = 1.0 / (y * (1.5 - (x2 * y * y)));
                    let diff = (delta_length - c.rest_length) / (delta_length * (i1 + i2));
                    self.particles[c.a].position = p1 + delta * i1 * diff;
                    self.particles[c.b].position = p2 - delta * i2 * diff;

                }

            }

        }

        if !any_particle_active {
            self.activity = self.activity.saturating_sub(1);
        }

    }

}

// ParticleSystem Templates ----------------------------------------------------
pub struct ParticleTemplate;
impl ParticleTemplate {

    pub fn schal(cols: usize, rows: usize, spacing: f32) -> ParticleSystem {

        let mut particles = ParticleSystem::new(cols * rows,  1);

        // Intialize particles
        particles.init(|i, p| {

            let row = i / cols;
            let col = i % cols;
            p.set_position(Vec2::new(
                (col as f32 * spacing) - cols as f32 * 0.5 * spacing,
                (row as f32 * spacing) - rows as f32 * 0.5 * spacing
            ));

            if i == 0 || i == cols - 1 {
                p.set_invmass(0.0);

            } else {
                p.set_invmass(0.90);
            }

        });


        // Intialize constraints
        for y in 0..rows {
            for x in 0..cols {

                // Constraints to lower right  _|
                let index = y * cols + x;

                if x < cols - 1 {
                    let right = y * cols + x + 1;
                    particles.add_constraint(ParticleConstraint::new(index, right, spacing));
                }

                if y < rows - 1 {
                    let bottom = (y + 1) * cols + x;
                    particles.add_constraint(ParticleConstraint::new(index, bottom, spacing));
                }

            }
        }

        particles

    }

}

