use ggez::graphics::{Color, DrawMode, DrawParam};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::*;
use std::cmp;
use std::collections::HashSet;
use std::env;

use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet};

const FRICTION: f32 = 0.08;
const BULLET_SPEED: f32 = 6.0;
const BULLET_SIZE: f32 = 8.0;
const BULLET_COLOR: (u8,u8,u8,u8) = (200, 200, 200, 255);

pub struct State {
    dt: std::time::Duration,
    velocity: Vector2,
    player: Player,
    bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    keys: HashSet<KeyCode>
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        let state = State {
            dt: std::time::Duration::new(0, 0),
            velocity: Vector2::new(0.0, 0.0),
            player: Player {
                position: Point2::new(50.0, 400.0),
                velocity: Vector2::new(0.0, 0.0),
                size: 30.0,
                bullet_counter: 0
            },
            bullets: vec!(),
            enemies: vec!(),
            keys: HashSet::with_capacity(6)
        };
        Ok(state)
    }
    fn handle_keys(&mut self, ctx: &mut Context) -> GameResult<()> {
        for key in &self.keys {
            match key {
                KeyCode::Up => {
                    self.player.velocity += Vector2::new(0.0, -1.0);
                }
                KeyCode::Left => {
                    self.player.velocity += Vector2::new(-1.0, 0.0);
                }
                KeyCode::Right => {
                    self.player.velocity += Vector2::new(1.0, 0.0);
                }
                KeyCode::Down => {
                    self.player.velocity += Vector2::new(0.0, 1.0);
                }
                KeyCode::Space => {
                    if self.player.bullet_counter == 0 {
                        self.bullets.push(Bullet {
                            position: self.player.position + Vector2::new(self.player.size/2.0 - BULLET_SIZE/2.0, -10.0),
                            velocity: Vector2::new(0.0, -BULLET_SPEED),
                            size: BULLET_SIZE
                        });
                        self.player.bullet_counter = 15;
                    }
                }
                KeyCode::Escape => {
                    event::quit(ctx);
                }
                _ => { /* Do nothing */ }
            }
        }
        Ok(())
    }
    fn move_bullets(&mut self, ctx: &mut Context) -> GameResult<()> {
        self.bullets = self.bullets.iter()
            .map(|bullet| {
                let mut b = bullet.clone();
                b.physics();
                b
            })
            .filter(|bullet| bullet.position[1] > 0.0) //remove off-screen buttons
            .collect();
        Ok(())
    }
    fn update_player(&mut self, ctx: &mut Context) -> GameResult<()> {
        //rudimentary physics
        self.player.position += self.player.velocity;
        self.player.velocity -= FRICTION*self.player.velocity;
        //clamp position to screen
        self.player.position[0] = self.player.position[0].max(0.0).min(640.0 - self.player.size);
        self.player.position[1] = self.player.position[1].max(0.0).min(480.0 - self.player.size);
        //weapon cooldown
        self.player.bullet_counter = (self.player.bullet_counter-1).max(0);
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        //self.dt = timer::delta(ctx);
        self.move_bullets(ctx);
        self.update_player(ctx);
        self.handle_keys(ctx);
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        let rect = graphics::Rect::new(self.player.position[0], self.player.position[1], self.player.size, self.player.size);
        let r1 = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;
        graphics::draw(ctx, &r1, DrawParam::default())?;

        for bullet in &mut self.bullets {
            let rect = graphics::Rect::new(bullet.position[0], bullet.position[1], bullet.size, bullet.size);
            let r1 = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, Color::from(BULLET_COLOR))?;
            graphics::draw(ctx, &r1, DrawParam::default())?;
        }

        graphics::present(ctx)?;
        Ok(())
    }
    fn key_up_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        self.keys.remove(&keycode);
    }
    fn key_down_event(&mut self, ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        self.keys.insert(keycode);
    }
}
