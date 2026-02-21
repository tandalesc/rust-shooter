mod config;
mod hitbox;
mod shooter;
mod spritesheet;
mod state;
mod weapon;

use std::env;
use std::path;
use ggez::*;

use crate::config::{DISPLAY_WIDTH, DISPLAY_HEIGHT};
use crate::state::State;

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("shooter_demo", "shishir")
        .window_setup(conf::WindowSetup::default().title("shooter_demo"))
        .window_mode(conf::WindowMode::default().dimensions(DISPLAY_WIDTH, DISPLAY_HEIGHT))
        .add_resource_path(resource_dir);
    let (mut ctx, event_loop) = cb.build()?;
    let game = State::new(&mut ctx)?;
    event::run(ctx, event_loop, game)
}
