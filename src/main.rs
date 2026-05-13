use macroquad::prelude::*;

use crowdsim_test::*;

const WORLD_SIZE: (u32, u32) = (1024, 768);
const UNIT_COUNT: usize = 64;

#[macroquad::main("App")]
async fn main() {
    miniquad::window::set_window_size(WORLD_SIZE.0, WORLD_SIZE.1);
    
    let mut world = World::new(WORLD_SIZE);
    let all_units: Vec<usize> = (0..UNIT_COUNT).collect();
    
    loop {
        if world.unit_count() < UNIT_COUNT {
            world.spawn_random_bih();
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            world.detask_units(&all_units);
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            let task = UnitTask::Walk { destination: mouse_position().into() };

            world.entask_units(&all_units, task);
        }
        
        world.update();
        
        world.draw();
        
        next_frame().await;
    }
}
