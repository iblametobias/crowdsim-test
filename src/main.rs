mod unit; mod world;

use macroquad::prelude::*;


const WORLD_SIZE: (u32, u32) = (1280, 720);
const UNIT_COUNT: usize = 64;

#[macroquad::main("App")]
async fn main() {
    miniquad::window::set_window_size(WORLD_SIZE.0, WORLD_SIZE.1);
    
    let mut world = world::World::new(WORLD_SIZE);

    let all: Vec<usize> = (0..UNIT_COUNT).collect();
    let squad_1: Vec<usize> = (0..UNIT_COUNT/2).collect();
    let squad_2: Vec<usize> = (UNIT_COUNT/2..UNIT_COUNT).collect();

    let mut selected = &squad_1;
    
    loop {
        if world.unit_count() < UNIT_COUNT {
            world.spawn_random_bih();
        }
        if is_key_pressed(KeyCode::Escape) {
            world.detask_units(&all);
        }
        if is_mouse_button_pressed(MouseButton::Right) {
            let task = unit::UnitTask::Walk { destination: mouse_position().into() };
            world.entask_units(selected, task);
        }
        if is_mouse_button_pressed(MouseButton::Left) {
            world.detask_units(selected);
        }

        if is_key_pressed(KeyCode::Key1) {
            selected = &squad_1;
        }
        if is_key_pressed(KeyCode::Key2) {
            selected = &squad_2;
        }
        
        world.update();
        
        world.draw();
        
        next_frame().await;
    }
}
