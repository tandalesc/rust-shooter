# Rust-Shooter
**By: Shishir Tandale** \
You can find the latest release on the *Releases* tab in GitHub.

---------
## Description
Rust-Shooter is designed to be an experiment in game design using the Rust
programming language. Currently it uses the [ggez](http://ggez.rs) game engine
-- a simple tool to lay the foundation on modern operating systems. Many of the
optimizations are hand-implemented to gain a better understanding of building
performant games. As a result, this is not designed to be a complete, user-
friendly game; there may be bugs or some features that do not work perfectly.

## Features
- Hundreds of bullets on screen at 60fps
- Upgradeable and switchable weapon types

## Interface
You can develop and build Rust-Shooter like any other Rust application.
### Development
- `cargo run` will build and run a test executable compiled with debug settings.
- `cargo build` will build a test executable at `./target/debug/rust-shooter`

### Build
- `cargo run --release` will build and run an optimized executable.
- `cargo build --release` will build an optimized executable at `./target/release/rust-shooter`
  - If you are compiling your own release, remember to copy the `./resources`
folder (and its contents) along with your final executable.
