mod unit; mod world;

use std::time::{Instant};

use macroquad::{prelude::*};

const WORLD_SIZE: (u32, u32) = (960, 720);
const UNIT_COUNT: usize = 512;

#[macroquad::main("SlotMap")]
async fn main() {
    miniquad::window::set_window_size(WORLD_SIZE.0, WORLD_SIZE.1);
    
    let mut world = world::World::new(WORLD_SIZE);
    let mut all = Vec::new();

    let mut update_times = Vec::new();

    prevent_quit();

    loop {
        if is_quit_requested() {
            let avg_update_time = update_times.iter().sum::<f32>() / update_times.len() as f32;
            println!("Average world update time: {:.2} ms", avg_update_time);
            break;
        }

        if world.unit_count() < UNIT_COUNT {
            all.push(world.spawn_test_unit());
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            let task = unit::UnitTask::Walk { destination: mouse_position().into() };
            world.entask_units(&all, task);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            world.detask_units(&all);
        }
        
        let t = Instant::now();
        world.update();
        let update_time_ms = t.elapsed().as_secs_f32() * 1000.0;
        update_times.push(update_time_ms);
        
        world.draw();
        
        let n = update_times.len().min(50);
        let avg_update_ms = update_times[update_times.len() - n..]
            .iter().sum::<f32>() / n as f32;

        draw_text(
            format!("World update time {:.2} ms", avg_update_ms).as_str(), 
            12.0, 32.0, 40.0, RED
        );
        
        next_frame().await;
    }
}
