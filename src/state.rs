use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::graphics::spritebatch::{SpriteBatch};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::*;
use uuid::Uuid;
use std::collections::HashSet;
use std::collections::HashMap;

use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet, BULLET_SPEED};

const BULLET_DAMAGE: f32 = 5.0;
const BULLET_SPACING_T: i16 = 12;
const BULLET_SPACING_X: f32 = 6.0;
const BULLET_SPACING_Y: f32 = 0.8;
const BULLET_OFFSET: f32 = -3.5;
const BULLET_SPREAD: i16 = 1;
const BULLET_ANGLE: f32 = 0.05;
const ENEMIES: (u8,u8) = (7, 3);

const HITBOX_COLOR: (f32, f32, f32, f32) = (1.0, 0.1, 0.1, 0.4);

pub const RESOLUTION: (f32, f32) = (640.0, 480.0);
pub const GRID_RESOLUTION: (f32, f32) = (10.0, 10.0);

const SHOW_FRAMERATE: bool = true;
const SHOW_HITBOXES: bool = false;

pub struct State {
    player: Player,
    bullets: HashMap<Uuid, Bullet>,
    enemies: HashMap<Uuid, Enemy>,
    keys: HashSet<KeyCode>,
    spritebatch_player: SpriteBatch,
    spritebatch_bullet: SpriteBatch,
    spritebatch_enemy: SpriteBatch
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        let mut state = State {
            player: Player::new(),
            bullets: HashMap::new(),
            enemies: HashMap::new(),
            keys: HashSet::with_capacity(6),
            spritebatch_player: SpriteBatch::new(Image::new(ctx, "/player.png").unwrap()),
            spritebatch_bullet: SpriteBatch::new(Image::new(ctx, "/bullet.png").unwrap()),
            spritebatch_enemy: SpriteBatch::new(Image::new(ctx, "/enemy.png").unwrap())
        };
        //generate a grid of enemies
        for x in 0..ENEMIES.0 {
            for y in 0..ENEMIES.1 {
                state.enemies.insert(
                    Uuid::new_v4(),
                    Enemy::new(
                        Point2::new(80.0, 50.0) +
                        (x as f32)*Vector2::new(70.0, 0.0) +
                        (y as f32)*Vector2::new(0.0, 70.0)
                    )
                );
            }
        }
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
                    if self.player.bullet_spacing==0 {
                        for _bullet_num in -BULLET_SPREAD..(BULLET_SPREAD+1) {
                            let bullet_num = (_bullet_num*2) as f32;
                            let velocity = Vector2::new(BULLET_ANGLE*bullet_num, -BULLET_SPEED);
                            let offset = Vector2::new(BULLET_SPACING_X*bullet_num, (BULLET_SPACING_Y*bullet_num).powf(2.0)+BULLET_OFFSET);
                            self.bullets.insert(Uuid::new_v4(), Bullet::new(&self.player, velocity, Some(offset)));
                        }
                        self.player.bullet_spacing = BULLET_SPACING_T;
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
    fn handle_bullets(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let mut bullet_ids: HashSet<Uuid> = HashSet::new();

        for (_, enemy) in &mut self.enemies {
            for (bullet_id, bullet) in &mut self.bullets {
                if bullet.hitbox_tree.collides_with(&enemy.hitbox_tree) {
                    //mark bullet for deletion
                    bullet_ids.insert(*bullet_id);
                    //do damage
                    enemy.health -= BULLET_DAMAGE;
                } else if bullet.position[0]<(-bullet.size) || bullet.position[0]>RESOLUTION.0 ||
                    bullet.position[1]<(-bullet.size) || bullet.position[1]>RESOLUTION.1 {
                    //mark bullet for deletion
                    bullet_ids.insert(*bullet_id);
                }
            }
        }

        //remove any bullets that were marked for deletion
        self.bullets.retain(|bullet_id, _| !bullet_ids.contains(bullet_id));
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if timer::ticks(ctx) % 50 == 0 {
            if SHOW_FRAMERATE {
                println!("Average FPS: {}, #Bullets: {}", timer::fps(ctx), self.bullets.len());
            }
        }

        self.handle_keys(ctx)?;
        self.handle_bullets(ctx)?;
        self.player.physics();
        for (_, enemy) in &mut self.enemies {
            enemy.physics();
        }
        for (_, bullet) in &mut self.bullets {
            bullet.physics();
        }

        //remove any enemies that died
        self.enemies.retain(|_, enemy| enemy.health > 0.0);
        //win state
        if self.enemies.len() == 0 {
            println!("\nyou win!\n");
            event::quit(ctx);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        //render bullets
        for (_, bullet) in &mut self.bullets {
            self.spritebatch_bullet.add(DrawParam::new().dest(bullet.position));
            if SHOW_HITBOXES {
                for hitbox_visit in bullet.hitbox_tree.bfs_iter() {
                    let hitbox = hitbox_visit.data;
                    let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from(HITBOX_COLOR))?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
        }
        //render enemies
        for (_, enemy) in &mut self.enemies {
            self.spritebatch_enemy.add(DrawParam::new().dest(enemy.position));
            if SHOW_HITBOXES {
                for hitbox_visit in enemy.hitbox_tree.bfs_iter() {
                    let hitbox = hitbox_visit.data;
                    let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from(HITBOX_COLOR))?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
        }
        //render player
        self.spritebatch_player.add(DrawParam::new().dest(self.player.position));
        if SHOW_HITBOXES {
            for hitbox_visit in self.player.hitbox_tree.bfs_iter() {
                let hitbox = hitbox_visit.data;
                let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from(HITBOX_COLOR))?;
                graphics::draw(ctx, &mesh, DrawParam::default())?;
            }
        }

        graphics::draw(ctx, &self.spritebatch_bullet, DrawParam::default())?;
        graphics::draw(ctx, &self.spritebatch_enemy, DrawParam::default())?;
        graphics::draw(ctx, &self.spritebatch_player, DrawParam::default())?;
        self.spritebatch_bullet.clear();
        self.spritebatch_enemy.clear();
        self.spritebatch_player.clear();

        graphics::present(ctx)?;
        Ok(())
    }
    fn key_up_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods) {
        self.keys.remove(&keycode);
    }
    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        self.keys.insert(keycode);
    }
}
