use std::{collections::VecDeque};

use macroquad::prelude::*;
use slotmap::new_key_type;

use crate::world::World;

const TASK_COMPLETION_RANGE: f32 = 32.0;

const MIN_SEPARATION_DIST: f32 = 0.00001;

const WEIGHT_COLLISION: f32 = 0.02;

const WEIGHT_SEPARATION: f32 = 4.0;
const WEIGHT_COHESION: f32 = 1.0;
const WEIGHT_ALIGNMENT: f32 = 0.0;
const WEIGHT_TASK: f32 = 0.8;

#[derive(Debug)]
pub struct Unit {
    position: Vec2,
    velocity: Vec2,
    tasks: VecDeque<TaskID>,
    stats: UnitStats,
}

impl Unit {
    pub fn new(position: Vec2, stats: UnitStats) -> Self {
        Self { position, velocity: Vec2::ZERO, stats: stats, tasks: VecDeque::new() }
    }
    
    // DESPERATELY needs to be refactored
    pub fn compute_update(&self, world: &World) -> UnitUpdateResult {
        let mut result = UnitUpdateResult::default();

        let current_taskid = self.get_current_taskid();
        
        let mut f_separation = Vec2::ZERO;
        
        let mut flock_center = Vec2::ZERO;
        let mut flock_direction =  Vec2::ZERO;
        let mut flock_size = 0u32;

        let mut total_stress = 0f32;
        let mut collisions_sum = Vec2::ZERO;
        
        for (_, unit) in world.get_units() {
            if unit == self { continue; }
            let diff = unit.position - self.position;
            let dir = diff.normalize();
            let dist = diff.length();

            let collision = (unit.stats.radius + self.stats.radius - dist) * 0.5;
            if collision > 0.0 {
                collisions_sum += -dir * collision;
                total_stress += collision;
            }

            if dist > self.stats.flocking_range { continue; }
            
            f_separation += -dir / (dist - self.stats.radius).max(MIN_SEPARATION_DIST);
            
            if current_taskid == unit.get_current_taskid() && current_taskid.is_some() {
                flock_size += 1;
                flock_center += unit.position;
                flock_direction += unit.velocity;
            }
        }

        result.collision_push = match total_stress {
            0.0 => Vec2::ZERO,
            n => collisions_sum / n * WEIGHT_COLLISION,
        };

        let f_cohesion = match flock_size {
            0 => Vec2::ZERO,
            n => (flock_center / n as f32 - self.position).normalize_or_zero(),
        };
        let f_alignment = flock_direction.normalize_or_zero();

        let mut f_task = Vec2::ZERO;
        
        match current_taskid.and_then( |&taskid| world.get_task(taskid).cloned() ) {
            Some(UnitTask::Walk { destination }) => {
                let diff = destination - self.position; // subtract owned from ref
                if diff.length() < TASK_COMPLETION_RANGE {
                    result.completed_tasks += 1;
                }
                
                f_task += diff.normalize();
            },
            None => {}
        }
        
        result.acceleration = (
            f_separation * WEIGHT_SEPARATION + 
            f_cohesion * WEIGHT_COHESION + 
            f_alignment * WEIGHT_ALIGNMENT +
            f_task * WEIGHT_TASK
        ).normalize_or_zero() * self.stats.acc; 
        result
    }

    pub fn apply_update(&mut self, update: UnitUpdateResult) {
        for _ in 0..update.completed_tasks { self.tasks.pop_front(); }
                
        let dt = get_frame_time();
        
        self.velocity += update.collision_push / dt; // physics, brometheus

        self.velocity += update.acceleration * dt;
        self.velocity /= 1.0 + self.stats.drag * dt;
        self.velocity = self.velocity.clamp_length_max(self.stats.speed);
        self.position += self.velocity * dt;

    }

    pub fn draw(&self) {
        draw_circle(
            self.position.x, self.position.y, self.stats.radius,
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
    }

    pub fn get_current_taskid(&self) -> Option<&TaskID> {
        self.tasks.front()
    }

    pub fn add_task(&mut self, taskid: TaskID) {
        self.tasks.push_back(taskid);
    }

    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
    }
}

// will fix this later
impl PartialEq for Unit {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.velocity == other.velocity
    }
}

#[derive(Debug)]
pub struct UnitStats {
    speed: f32,
    acc: f32,
    drag: f32,
    radius: f32,
    flocking_range: f32,
}

impl UnitStats {
    pub const TEST0: Self = Self {
        speed: 120.0, acc: 500.0, drag: 6.0, radius: 5.0, flocking_range: 24.0
    };
}

#[derive(Clone, Copy)]
pub enum UnitTask {
    Walk { destination: Vec2 }
}

#[derive(Default, Debug)]
pub struct UnitUpdateResult {
    completed_tasks: u8,
    acceleration: Vec2,
    collision_push: Vec2,
}

new_key_type! { pub struct UnitID; }
new_key_type! { pub struct TaskID; }