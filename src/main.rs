extern crate nalgebra;

mod state;
mod shooter;

use ggez::*;
use std::path;

use crate::state::State;


fn main() -> GameResult {
    let cb = ContextBuilder::new("shooter_demo", "shishir")
        .window_setup(conf::WindowSetup::default().title("shooter_demo"))
        .window_mode(conf::WindowMode::default().dimensions(640.0, 480.0));
        //.add_resource_path(resource_dir);
    let (ctx, events_loop) = &mut cb.build()?;
    let game = &mut State::new(ctx)?;
    event::run(ctx, events_loop, game)
}
