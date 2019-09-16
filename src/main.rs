extern crate nalgebra;
extern crate rand;

mod state;
mod shooter;

use std::env;
use std::path;
use ggez::*;

use crate::state::{State, RESOLUTION};

fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("shooter_demo", "shishir")
        .window_setup(conf::WindowSetup::default().title("shooter_demo"))
        .window_mode(conf::WindowMode::default().dimensions(RESOLUTION.0, RESOLUTION.1))
        .add_resource_path(resource_dir);
    let (ctx, events_loop) = &mut cb.build()?;
    let game = &mut State::new(ctx)?;
    event::run(ctx, events_loop, game)
}
