mod unit; mod world;

use std::time::{Instant};

use macroquad::{prelude::*, ui::{hash, root_ui}};

use crate::unit::*;

const WORLD_SIZE: (u32, u32) = (960, 720);
const UNIT_COUNT: usize = 128;


#[macroquad::main("SlotMap")]
async fn main() {
    miniquad::window::set_window_size(WORLD_SIZE.0, WORLD_SIZE.1);
    
    let mut world = world::World::new(WORLD_SIZE);
    let mut all = Vec::new();

    let mut update_times = Vec::new();
    let mut last_update_time_ms = 0f32;
    let mut last_text_update = Instant::now();

    prevent_quit();
    
    for _ in 0..UNIT_COUNT { all.push(world.spawn_test_unit()); }

    loop {
        if is_quit_requested() {
            let avg_update_time = update_times
                .iter().sum::<f32>() / update_times.len() as f32;
            println!("Average world update time: {:.2} ms", avg_update_time);
            break;
        }

        if is_mouse_button_pressed(MouseButton::Right) {
            let task = Task::Walk { destination: mouse_position().into() };
            world.entask_units(&all, task);
        }
        // if is_mouse_button_pressed(MouseButton::Left) {
        if is_key_pressed(KeyCode::Escape) {
            world.detask_units(&all);
        }
        
        let t = Instant::now();
        world.update();
        let update_time_ms = t.elapsed().as_secs_f32() * 1000.0;
        update_times.push(update_time_ms);

        if last_text_update.elapsed().as_millis() > 200 {
            last_text_update = Instant::now();
            last_update_time_ms = update_time_ms;
        } 

        world.draw();

        root_ui().window(hash!(), vec2(12.0, 100.0), vec2(360.0, 120.0), |ui| {
            CONSTANTS.with_borrow_mut(|c| {
                ui.slider(hash!(), "PREDICTION_TIME", 0.0..4.0, &mut c.PREDICTION_TIME);
                ui.slider(hash!(), "AVOID_MARGIN", 0.0..64.0, &mut c.AVOID_MARGIN);
                ui.slider(hash!(), "AVOID_WEIGHT", 0.0..12.0, &mut c.AVOID_WEIGHT);
            });
        });
        
        draw_text(
            format!("World update time {:.2} ms", last_update_time_ms).as_str(), 
            12.0, 32.0, 40.0, RED
        );

        
        
        next_frame().await;
    }
}
