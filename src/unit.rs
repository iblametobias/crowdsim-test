use std::{cell::RefCell, collections::VecDeque};

use macroquad::prelude::*;
use slotmap::new_key_type;

use crate::world::World;

thread_local! {
    pub static CONSTANTS: RefCell<Constants> = RefCell::new(Constants::default());
}

#[allow(non_snake_case)]
#[derive(Clone, Copy)]
pub struct Constants {
    pub TASK_COMPLETION_RANGE: f32,
    pub UNIT_VIEW_DIST: f32,
    pub PREDICTION_TIME: f32,
    pub AVOID_MARGIN: f32,
    pub AVOID_WEIGHT: f32,
}

impl Default for Constants {
    fn default() -> Self {
        Self {
            TASK_COMPLETION_RANGE: 64.0,
            UNIT_VIEW_DIST: 64.0,
            PREDICTION_TIME: 1.0,
            AVOID_MARGIN: 8.0,
            AVOID_WEIGHT: 3.0,
        }
    }
}

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

    pub fn compute_update(&self, self_id: UnitID, world: &World) -> UnitUpdateResult {
        let mut result = UnitUpdateResult::default();

        let mut avoidance = Vec2::ZERO;

        let (UNIT_VIEW_DIST, PREDICTION_TIME, AVOID_MARGIN, AVOID_WEIGHT) = 
            CONSTANTS.with_borrow(|c| { (c.UNIT_VIEW_DIST, c.PREDICTION_TIME, c.AVOID_MARGIN, c.AVOID_WEIGHT)
        });

        for unit in world.get_units_near(self_id, UNIT_VIEW_DIST) {
            let rel_pos = unit.position - self.position;
            let rel_vel = unit.velocity - self.velocity;

            let rel_speed_sq = rel_vel.length_squared();

            if rel_speed_sq > 0.0001 {
                let t = (-rel_pos.dot(rel_vel) / rel_speed_sq)
                    .clamp(0.0, PREDICTION_TIME);

                let future_offset = rel_pos + rel_vel * t;

                let safe_dist = self.stats.radius + unit.stats.radius + AVOID_MARGIN;

                let future_dist = future_offset.length();

                if future_dist < safe_dist {
                    let strength = 1.0 - future_dist / safe_dist;
                    avoidance += -future_offset.normalize_or_zero() * strength;
                }
            }
        }

        let task_movement = if let Some(task_result) = self.get_current_taskid()
            .and_then( |&taskid| world.get_task(taskid))
            .map( |&task| task.get_task_result(self)) {
                result.completed_tasks += task_result.completed as u8;
                task_result.move_direction
        } else { Vec2::ZERO };
    
        result.movement = task_movement + avoidance * AVOID_WEIGHT;

        result
    }

    pub fn apply_update(&mut self, update: UnitUpdateResult) {
        for _ in 0..update.completed_tasks { self.tasks.pop_front(); }
                
        let dt = get_frame_time();
        

        self.velocity += update.movement.clamp_length_max(1.0) * self.stats.acc * dt;
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

    pub fn distance_squared(&self, other: &Self) -> f32 {
        (self.position - other.position).length_squared()
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
        speed: 120.0, acc: 500.0, drag: 6.0, radius: 5.0, flocking_range: 64.0
    };
}

#[derive(Clone, Copy)]
pub enum Task {
    Walk { destination: Vec2 }
}

impl Task {
    fn get_task_result(&self, unit: &Unit) -> TaskResult {
        let TASK_COMPLETION_RANGE = CONSTANTS.with_borrow(|c| {c.TASK_COMPLETION_RANGE});
        match self {
            &Self::Walk { destination } => {
                let mut result = TaskResult::default();
                let diff = destination - unit.position;

                if diff.length() < unsafe{TASK_COMPLETION_RANGE} {
                    result.completed = true;
                } else {
                    result.move_direction = diff.normalize();
                }

                result
            }
        }
    }
}

#[derive(Default)]
struct TaskResult {
    completed: bool,
    move_direction: Vec2,
}

#[derive(Default, Debug)]
pub struct UnitUpdateResult {
    movement: Vec2,
    completed_tasks: u8,
}

new_key_type! { pub struct UnitID; }
new_key_type! { pub struct TaskID; }