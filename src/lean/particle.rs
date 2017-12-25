// Copyright (c) 2017 Ivo Wetzel

// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// Internal Dependencies ------------------------------------------------------
use super::Vec2;


// Traits ---------------------------------------------------------------------
pub trait Constraint {
    fn first_particle(&self) -> usize;
    fn second_particle(&self) -> usize;
    fn solve(&self, &mut [Particle]) {}
    fn visible(&self) -> bool {
        false
    }
}

pub trait ParticleSystemLike {
    fn get_particles(&self) -> Vec<Particle>;
    fn get_constraints(&self) -> Vec<Box<Constraint>>;
}

pub trait ParticleLike {
    fn to_constaint(&self) -> Option<Box<Constraint>> where Self: Sized;
    fn to_particle(&self) -> Particle where Self: Sized;
}

// 2D Particles Constraints ---------------------------------------------------
pub struct StickConstraint {
    a: usize,
    b: usize,
    rest_length: f32,
    visible: bool
}

impl StickConstraint {

    pub fn new(a: usize, b: usize, rest_length: f32) -> Self {
        Self {
            a,
            b,
            rest_length,
            visible: false
        }
    }

    pub fn set_visible(&mut self, visual: bool) {
        self.visible = visual;
    }

}

impl Constraint for StickConstraint {

    fn visible(&self) -> bool {
        self.visible
    }

    fn first_particle(&self) -> usize {
        self.a
    }

    fn second_particle(&self) -> usize {
        self.b
    }

    fn solve(&self, particles: &mut [Particle]) {

        let i1 = particles[self.a].inv_mass;
        let i2 = particles[self.b].inv_mass;

        if i1 + i2 > 0.0 {

            let p1 = particles[self.a].position;
            let p2 = particles[self.b].position;
            let delta = p2 - p1;

            // Fast inverse square root
            let dot = delta * delta;
            let x2 = dot * 0.5;
            let x = 0x5f37_5a86 - (dot.to_bits() >> 1);
            let y = f32::from_bits(x);
            let delta_length = 1.0 / (y * (1.5 - (x2 * y * y)));
            let diff = (delta_length - self.rest_length) / (delta_length * (i1 + i2));
            particles[self.a].position = p1 + delta * i1 * diff;
            particles[self.b].position = p2 - delta * i2 * diff;

        }

    }

}

/*
pub struct AngularConstraint {
    a: usize,
    b: usize,
    angle: f32
}

impl AngularConstraint {

    pub fn new(a: usize, b: usize, angle: f32) -> Self {
        Self {
            a,
            b,
            angle
        }
    }

}

impl Constraint for AngularConstraint {

    fn visible(&self) -> bool {
        false
    }

    fn first_particle(&self) -> usize {
        self.a
    }

    fn second_particle(&self) -> usize {
        self.b
    }

    fn solve(&self, particles: &mut [Particle]) {

        let top = particles[self.a].position;
        let bot = particles[self.b].position;
        let da = top.angle_between(bot); // TODO does this return -PI to PI ?
        if da > self.angle {

            // TODO need length on particle
            // TODO we need to unify the bones with the particles!
            // let l = particles[self.b].len();

            /*
            float l = bot.Length();   // store length of wrist
            bot = top.UnitVector();   // copy orientation
            bot.Mult(-l);             // scale to original length

                // difference of where it is, and where it should be:
            Vector diff = t3.pos - (t2.pos+bot);

                // scale it to half length:
            diff.Mult(0.5);

            // give knee and foot one push each in opposite dirs:
            t3.pos = t3.pos - diff;
            t2.pos = t2.pos + diff;
            */

        }

    }

}
*/

// 2D Particle Abstraction ----------------------------------------------------
#[derive(Default, Debug, Copy, Clone)]
pub struct Particle {
    pub position: Vec2,
    pub prev_position: Vec2,
    rest_position: Vec2,
    constant_force: Vec2,
    acceleration: Vec2,
    inv_mass: f32
}

impl Particle {


    pub fn new(position: Vec2) -> Self {
        Self {
            position: position,
            prev_position: position,
            rest_position: position,
            constant_force: Vec2::zero(),
            acceleration: Vec2::zero(),
            inv_mass: 1.0
        }
    }

    pub fn with_inv_mass(position: Vec2, inv_mass: f32) -> Self {
        Self {
            position: position,
            prev_position: position,
            rest_position: position,
            constant_force: Vec2::zero(),
            acceleration: Vec2::zero(),
            inv_mass: inv_mass
        }
    }

    pub fn set_invmass(&mut self, mass: f32) {
        self.inv_mass = mass;
    }

    pub fn set_position(&mut self, p: Vec2) {
        self.position = p;
        self.prev_position = p;
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.position = self.position + force;
    }

    pub fn at_rest(&mut self) -> bool {
        if (self.position - self.rest_position).len().abs() > 0.1 {
            self.rest_position = self.position;
            false

        } else {
            true
        }
    }

    /*
    pub fn apply_constant_force(&mut self, force: Vec2) {
        self.constant_force = force;
    }*/

}



// Simple Verlet based Particle System ----------------------------------------
pub struct ParticleSystem {
    particles: Vec<Particle>,
    constraints: Vec<Box<Constraint>>,
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

    /*
    pub fn from<T: ParticleSystemLike>(system_like: &T, iterations: usize) -> ParticleSystem {
        Self {
            particles: system_like.get_particles(),
            constraints: system_like.get_constraints(),
            iterations,
            activity: 10
        }
    }*/

    // Getters ----------------------------------------------------------------
    pub fn active(&self) -> bool {
        self.activity > 0
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Particle {
        &mut self.particles[index]
    }

    // Methods ----------------------------------------------------------------
    pub fn activate(&mut self) {
        self.activity = 10;
    }

    pub fn add_constraint<T: Constraint + 'static>(&mut self, constraint: T) {
        self.constraints.push(Box::new(constraint));
        self.activate();
    }

    pub fn step<C: Fn(&mut Particle)>(&mut self, time_step: f32, gravity: Vec2, collider: C) {
        if self.active() {
            ParticleSystem::accumulate_forces(gravity, &mut self.particles[..]);
            ParticleSystem::verlet(time_step, &mut self.particles[..]);
            if !ParticleSystem::satisfy_constraints(self.iterations, &mut self.particles[..], &self.constraints[..], collider) {
                self.activity = self.activity.saturating_sub(1);
            }
        }
    }

    // Visitors ---------------------------------------------------------------
    /*
    pub fn visit_particles<C: FnMut(usize, &Particle)>(&self, mut callback: C) {
        for (index, p) in self.particles.iter().enumerate() {
            callback(index, p);
        }
    }*/

    pub fn visit_particles_mut<C: FnMut(usize, &mut Particle)>(&mut self, mut callback: C) {
        for (index, p) in self.particles.iter_mut().enumerate() {
            callback(index, p);
        }
    }

    pub fn visit_particles_chained<C: FnMut(usize, &Particle, &Particle)>(&mut self, mut callback: C) {
        for i in 1..self.particles.len() {
            callback(i - 1, &self.particles[i - 1], &self.particles[i]);
        }
    }

    pub fn visit_constraints<C: FnMut((usize, Vec2), (usize, Vec2), bool)>(&self, mut callback: C) {
        for constraint in &self.constraints {
            let a = self.particles[constraint.first_particle()].position;
            let b = self.particles[constraint.second_particle()].position;
            callback(
                (constraint.first_particle(), a),
                (constraint.second_particle(), b),
                constraint.visible()
            );
        }
    }

    // Internal ---------------------------------------------------------------
    pub fn verlet(time_step: f32, particles: &mut [Particle]) {
        for p in particles {
            let current_pos = p.position;
            let change = p.position - p.prev_position + p.acceleration * time_step * time_step;
            p.position = p.position + change * p.inv_mass;
            p.prev_position = current_pos;
        }
    }

    pub fn accumulate_forces(gravity: Vec2, particles: &mut [Particle]) {
        for p in particles {
            p.acceleration = gravity + p.constant_force;
        }
    }

    pub fn satisfy_constraints<C: Fn(&mut Particle)>(
        iterations: usize,
        particles: &mut [Particle],
        constraints: &[Box<Constraint>],
        collider: C
    ) -> bool {

        let mut any_particle_active = false;
        for _ in 0..iterations {

            for mut p in particles.iter_mut() {
                collider(&mut p);
                if !p.at_rest() {
                    any_particle_active = true;
                }
            }

            for c in constraints {
                c.solve(particles);
            }

        }

        any_particle_active

    }

}



// ParticleSystem Templates ----------------------------------------------------
pub struct ParticleTemplate;
impl ParticleTemplate {

    pub fn schal(cols: usize, rows: usize, spacing: f32, position: Vec2) -> ParticleSystem {

        let mut particles = ParticleSystem::new(cols * rows,  2);

        // Intialize particles
        particles.visit_particles_mut(|i, p| {

            p.set_position(position);

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
                    particles.add_constraint(StickConstraint::new(index, right, spacing));
                }

                if y < rows - 1 {
                    let bottom = (y + 1) * cols + x;
                    particles.add_constraint(StickConstraint::new(index, bottom, spacing));
                }

            }
        }

        particles

    }

}

