use std::collections::{HashMap, VecDeque};

use macroquad::prelude::*;

const TASK_COMPLETION_RANGE: f32 = 32.0;
const WEIGHT_TASK: f32 = 2.0;
const WEIGHT_COHESION: f32 = 0.2;
const WEIGHT_SEPARATION: f32 = 16.0;

pub struct World {
    units: HashMap<usize, Unit>,
    last_unit_id: usize,
    tasks: HashMap<usize, UnitTask>,
    last_task_id: usize, 
    rng: rand::RandGenerator,
    size: (u32, u32),
}

impl World {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            units: HashMap::new(), last_unit_id: 0, 
            tasks: HashMap::new(), last_task_id: 0,
            rng: rand::RandGenerator::new(), size
        }
    }

    pub fn spawn_unit(&mut self, unit: Unit) {
        self.units.insert(self.last_unit_id, unit);
        self.last_unit_id += 1;
    }

    pub fn spawn_random_bih(&mut self) {
        let result = self.units.insert(
            self.last_unit_id, Unit::new(self.last_unit_id, Vec2 { 
                x: self.rng.gen_range(0.0, self.size.0 as f32), 
                y: self.rng.gen_range(0.0, self.size.1 as f32) }
            )
        );

        if let Some(_) = result {
            panic!("Unit {} replaced with sum other bih", self.last_unit_id)
        }
        self.last_unit_id += 1;
    }

    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    pub fn entask_units(&mut self, units: &Vec<usize>, task: UnitTask) {
        self.tasks.insert(self.last_task_id, task);
        for id in units {
            if let Some(unit) = self.units.get_mut(id) {
                unit.add_task(self.last_task_id);
            }
        }

        self.last_task_id += 1;
    }

    pub fn detask_units(&mut self, units: &Vec<usize>) {
        for id in units {
            if let Some(unit) = self.units.get_mut(id) {
                unit.clear_tasks();
            }
        }
    }

    pub fn update(&mut self) {
        let update_results: Vec<(usize, UnitUpdateResult)> = self.units.iter()
            .map(
                |(&id, unit)| { (id, unit.compute_update(&self)) }
            ).collect();

        for (id, update) in update_results {
            if let Some(unit) = self.units.get_mut(&id) {
                unit.apply_update(update);
            }
        }
    }

    pub fn draw(&self) {
        draw_rectangle(
            0.0, 0.0, self.size.0 as f32, self.size.1 as f32, 
            Color::new(0.1, 0.1, 0.1, 1.0)
        );
        for unit in self.units.values() {
            unit.draw();
        }
    }
}

struct UnitStats {
    speed: f32,
    acc: f32,
    drag: f32,
    radius: f32,
}

impl UnitStats {
    const CUNT1: Self = Self {
        speed: 50.0, acc: 100.0, drag: 0.3, radius: 5.0
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
        let taskid = self.get_current_taskid();
        
        let mut f_separation = Vec2::ZERO;
        
        let mut flock_center = Vec2::ZERO;
        let mut flock_size = 0u32;
        
        for (&id, unit) in &world.units {
            if id == self.id { continue; }
            
            let diff = unit.position - self.position;
            if diff.length() < self.stats.radius * 3.0 {
                f_separation += -diff / diff.length_squared();
            }
            
            if diff.length() < 32.0 && taskid.is_some() && taskid == unit.get_current_taskid() {
                flock_size += 1;
                flock_center += unit.position;
            }
        }

        let f_cohesion = if flock_size == 0 { Vec2::ZERO } 
                       else { flock_center / flock_size as f32 - self.position };
        
        let mut f_task = Vec2::ZERO;
        
        if let Some(taskid) = self.tasks.front() {
            match world.tasks[taskid] {
                UnitTask::Walk { destination } => {
                    let diff = destination - self.position;
                    if diff.length() < TASK_COMPLETION_RANGE { return UnitUpdateResult::TaskCompleted; }
                    
                    f_task += diff.normalize();
                }
            }
        }
        
        UnitUpdateResult::Accelerate(
            (
                f_separation * WEIGHT_SEPARATION + 
                f_cohesion * WEIGHT_COHESION + 
                f_task * WEIGHT_TASK
            ).normalize_or_zero() * get_frame_time() * self.stats.acc
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