use macroquad::prelude::*;
use slotmap::SlotMap;

use crate::unit::*;

pub struct World {
    units: SlotMap<UnitID, Unit>,
    tasks: SlotMap<TaskID, Task>,
    rng: rand::RandGenerator,
    size: (u32, u32),
}

impl World {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            units: SlotMap::with_key(), 
            tasks: SlotMap::with_key(),
            rng: rand::RandGenerator::new(),
            size: size
        }
    }

    pub fn get_units(&self) -> &SlotMap<UnitID, Unit> {
        &self.units
    }

    pub fn get_units_near(&self, unitid: UnitID, max_distance: f32) -> impl Iterator<Item = &Unit> {
        let unit = self.units.get(unitid).unwrap();
        let max_distance_squared = max_distance * max_distance;

        self.units.iter()
            .filter(
                move |&(id, u)| {
                    u.distance_squared(unit) <= max_distance_squared && id != unitid
                }
            )
            .map(|(_, u)| u)
    }

    pub fn spawn_unit(&mut self, unit: Unit) -> UnitID {
        self.units.insert(unit)
    }

    pub fn spawn_test_unit(&mut self) -> UnitID {
        let unit = Unit::new(
            Vec2 { 
            x: self.rng.gen_range(0.0, self.size.0 as f32), 
            y: self.rng.gen_range(0.0, self.size.1 as f32) },
            UnitStats::TEST0
        );

        self.spawn_unit(unit)
    }

    pub fn unit_count(&self) -> usize {
        self.units.len()
    }

    pub fn entask_units(&mut self, units: &[UnitID], task: Task) {
        let taskid = self.tasks.insert(task);
        for &id in units {
            if let Some(unit) = self.units.get_mut(id) {
                unit.add_task(taskid);
            }
        }
    }

    pub fn detask_units(&mut self, units: &[UnitID]) {
        for &id in units {
            if let Some(unit) = self.units.get_mut(id) {
                unit.clear_tasks();
            }
        }
    }

    pub fn get_task(&self, taskid: TaskID) -> Option<&Task> {
        self.tasks.get(taskid)
    }

    pub fn update(&mut self) {
        let update_results: Vec<(UnitID, UnitUpdateResult)> = self.units.iter()
            .map(
                |(id, unit)| { (id, unit.compute_update(id, &self)) }
            ).collect();

        for (id, update) in update_results {
            if let Some(unit) = self.units.get_mut(id) {
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