#![feature(exclusive_range_pattern)]

#[macro_use]
mod utils;
mod ui;
mod game;
mod engine;

use std::mem;

use wasm_bindgen::prelude::*;
use web_sys;
use js_sys::Math;

#[wasm_bindgen]
pub fn start() {
    utils::set_panic_hook();

    let config = game::Config {
        update_interval: 60,
    };

    let rows = 100;
    let cols = 100;

    let window = web_sys::window().expect("There must be a window instance");
    let document = window.document().expect("There must be a document instance");
    let canvas = ui::Canvas::new(&document, rows, cols);

    let mut world = engine::World::new(rows, cols);
    generate_initial_world(&mut world);

    let state = game::State::new(canvas, world, config);
    state.resume();

    // TODO: We really should manage the memory correctly!
    mem::forget(state);
}

fn generate_initial_world(world: &mut engine::World) {
    for col in 0..(world.height()) {
        for row in 0..(world.width()) {
            if Math::random() < 0.3 {
                world.set_cell(row, col, engine::Cell::Alive);
            }
        }
    }
}