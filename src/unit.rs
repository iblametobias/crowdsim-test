use std::{collections::VecDeque};

use macroquad::prelude::*;

use crate::world::World;

const TASK_COMPLETION_RANGE: f32 = 32.0;
const WEIGHT_SEPARATION: f32 = 5.0;
const WEIGHT_COHESION: f32 = 0.5;
const WEIGHT_ALIGNMENT: f32 = 1.5;
const WEIGHT_TASK: f32 = 2.0;

pub struct Unit {
    id: usize,
    position: Vec2,
    velocity: Vec2,
    tasks: VecDeque<usize>,
    stats: UnitStats,
}

impl Unit {
    pub fn new(id: usize, position: Vec2) -> Self {
        Self { id, position, velocity: Vec2::ZERO, stats: UnitStats::CUNT1, tasks: VecDeque::new() }
    }
    
    pub fn compute_update(&self, world: &World) -> UnitUpdateResult {
        let current_taskid = self.get_current_taskid();
        
        let mut f_separation = Vec2::ZERO;
        
        let mut flock_center = Vec2::ZERO;
        let mut flock_direction =  Vec2::ZERO;
        let mut flock_size = 0u32;
        
        for (&id, unit) in world.get_units() {
            let diff = unit.position - self.position;
            if diff.length() > self.stats.flocking_range || id == self.id { continue; }
            
            f_separation += -diff / diff.length_squared();
            
            if current_taskid == unit.get_current_taskid() && current_taskid.is_some() {
                flock_size += 1;
                flock_center += unit.position;
                flock_direction += unit.velocity;
            }
        }

        let f_cohesion = match flock_size {
            0 => Vec2::ZERO,
            n => (flock_center / n as f32 - self.position).normalize_or_zero(),
        };
        let f_alignment = flock_direction.normalize_or_zero();

        let mut f_task = Vec2::ZERO;
        
        match world.try_get_task(current_taskid) {
            Some(UnitTask::Walk { destination }) => {
                let diff = destination - self.position;
                if diff.length() < TASK_COMPLETION_RANGE { return UnitUpdateResult::TaskCompleted; }
                
                f_task += diff.normalize();
            },
            None => {}
        }
        
        UnitUpdateResult::Accelerate(
            (
                f_separation * WEIGHT_SEPARATION + 
                f_cohesion * WEIGHT_COHESION + 
                f_alignment * WEIGHT_ALIGNMENT +
                f_task * WEIGHT_TASK
            ).clamp_length_max(1.0) * get_frame_time() * self.stats.acc
        )
    }
    
    pub fn apply_update(&mut self, update: UnitUpdateResult) {
        match update {
            UnitUpdateResult::TaskCompleted => { self.tasks.pop_front(); },
            UnitUpdateResult::Accelerate(acc) => { self.velocity += acc; }
        }
        
        self.velocity *= 1.0 - self.stats.drag;
        self.velocity = self.velocity.clamp_length_max(self.stats.speed);
        self.position += self.velocity;
    }

    pub fn draw(&self) {
        draw_circle(
            self.position.x, self.position.y, self.stats.radius,
            Color::new(1.0, 1.0, 1.0, 1.0)
        );
    }

    pub fn get_current_taskid(&self) -> Option<&usize> {
        self.tasks.front()
    }

    pub fn add_task(&mut self, taskid: usize) {
        self.tasks.push_back(taskid);
    }

    pub fn clear_tasks(&mut self) {
        self.tasks.clear();
    }
}

struct UnitStats {
    speed: f32,
    acc: f32,
    drag: f32,
    radius: f32,
    flocking_range: f32,
}

impl UnitStats {
    const CUNT1: Self = Self {
        speed: 50.0, acc: 100.0, drag: 0.3, radius: 5.0, flocking_range: 20.0
    };
}

#[derive(Clone, Copy)]
pub enum UnitTask {
    Walk { destination: Vec2 }
}

pub enum UnitUpdateResult {
    TaskCompleted,
    Accelerate(Vec2)
}
