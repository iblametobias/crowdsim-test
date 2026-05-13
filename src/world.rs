use std::collections::HashMap;

use macroquad::prelude::*;

use crate::unit::*;

pub struct World {
    units: HashMap<usize, Unit>,
    last_unit_id: usize,
    pub tasks: HashMap<usize, UnitTask>,
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


    pub fn try_get_task(&self, taskid: Option<&usize>) -> Option<UnitTask> {
        taskid.map( |id| { self.tasks[id] } )
    }

    pub fn get_units(&self) -> &HashMap<usize, Unit> {
        &self.units
    }

    pub fn spawn_unit(&mut self, unit: Unit) {
        self.units.insert(self.last_unit_id, unit);
        self.last_unit_id += 1;
    }

    pub fn spawn_random_bih(&mut self) {
        let None = self.units.insert(
            self.last_unit_id, Unit::new(self.last_unit_id, Vec2 { 
                x: self.rng.gen_range(0.0, self.size.0 as f32), 
                y: self.rng.gen_range(0.0, self.size.1 as f32) }
            )
        ) else {
            panic!("Unit {} replaced with sum other bih", self.last_unit_id)
        };
 

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

