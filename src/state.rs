use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::graphics::spritebatch::{SpriteBatch};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::*;
use rand::Rng;
use rand::rngs::ThreadRng;
use uuid::Uuid;
use std::collections::HashSet;
use std::collections::HashMap;

use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet, GameObject};

const BULLET_DAMAGE: f32 = 5.0;
const BULLET_SPACING_T: i16 = 12;
const BULLET_SPACING_X: f32 = 5.0;
const BULLET_SPACING_Y: f32 = 0.7;
const BULLET_OFFSET: f32 = -3.0;
const BULLET_SPREAD: i16 = 2;
const BULLET_ANGLE: f32 = 0.3;
pub const BULLET_SPEED: f32 = 5.0;
pub const BULLET_SIZE: f32 = 10.0;

const ENEMIES: (u8,u8) = (7, 3);
const ENEMY_SHOOT_CHANCE: usize = 50;

const PLAYER_INVINCIBILITY: i16 = 60;
const HITBOX_COLOR: (f32, f32, f32, f32) = (1.0, 0.1, 0.1, 0.4);
pub const FRICTION: f32 = 0.1;

pub const RESOLUTION: (f32, f32) = (640.0, 480.0);
pub const GRID_RESOLUTION: (f32, f32) = (10.0, 10.0);

const SHOW_FRAMERATE: bool = false;
const SHOW_HITBOXES: bool = false;

pub struct State {
    player: Player,
    bullets: HashMap<Uuid, Bullet>,
    enemy_bullets: HashMap<Uuid, Bullet>,
    bullet_ids: HashSet<Uuid>,
    enemy_bullet_ids: HashSet<Uuid>,
    enemies: HashMap<Uuid, Enemy>,
    keys: HashSet<KeyCode>,
    rng: ThreadRng,
    spritebatch_player: SpriteBatch,
    spritebatch_bullet: SpriteBatch,
    spritebatch_enemy: SpriteBatch
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        let mut state = State {
            player: Player::new(),
            bullets: HashMap::new(),
            enemy_bullets: HashMap::new(),
            bullet_ids: HashSet::new(),
            enemy_bullet_ids: HashSet::new(),
            enemies: HashMap::new(),
            rng: rand::thread_rng(),
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
        //check player bullets and enemies for collisions
        for (_, enemy) in &mut self.enemies {
            for (bullet_id, bullet) in &self.bullets {
                if bullet.collides_with(enemy) {
                    //mark bullet for deletion
                    self.bullet_ids.insert(*bullet_id);
                    //do damage
                    enemy.health -= BULLET_DAMAGE;
                    enemy.flash_frames = 5;
                } else if bullet.position[0]<(-bullet.size) || bullet.position[0]>RESOLUTION.0 ||
                    bullet.position[1]<(-bullet.size) || bullet.position[1]>RESOLUTION.1 {
                    //mark bullet for deletion
                    self.bullet_ids.insert(*bullet_id);
                }
            }
        }
        //check enemy bullets and player for collisions
        for (bullet_id, bullet) in &self.enemy_bullets {
            if self.player.invincibility_frames==0 && bullet.collides_with(&self.player) {
                self.enemy_bullet_ids.insert(*bullet_id);
                self.player.health -= 10.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            } else if bullet.position[0]<(-bullet.size) || bullet.position[0]>RESOLUTION.0 ||
                bullet.position[1]<(-bullet.size) || bullet.position[1]>RESOLUTION.1 {
                self.enemy_bullet_ids.insert(*bullet_id);
            }
        }
        //remove any bullets that were marked for deletion
        for id in &self.enemy_bullet_ids { self.enemy_bullets.remove(id); }
        for id in &self.bullet_ids { self.bullets.remove(id); }
        self.enemy_bullet_ids.clear();
        self.bullet_ids.clear();
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if timer::ticks(ctx) % 50 == 0 {
            if SHOW_FRAMERATE {
                println!("Average FPS: {}, #Bullets: {}", timer::fps(ctx), self.bullets.len()+self.enemy_bullets.len());
            }
        }

        self.handle_keys(ctx)?;
        self.handle_bullets(ctx)?;
        self.player.physics();
        let num_enemies = self.enemies.len();
        for (_, enemy) in &mut self.enemies {
            enemy.physics();
            if self.rng.gen_range(0, ENEMY_SHOOT_CHANCE*num_enemies)==0 {
                let offset = Vector2::new(0.0, 20.0);
                //bullets go towards player
                let velocity = (self.player.position - enemy.position).normalize()*BULLET_SPEED;
                self.enemy_bullets.insert(Uuid::new_v4(), Bullet::new(enemy, velocity, Some(offset)));
            }
            if self.player.invincibility_frames==0 && enemy.collides_with(&self.player) {
                self.player.health -= 20.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            }
        }
        for (_, bullet) in &mut self.bullets {
            bullet.physics();
        }
        for (_, bullet) in &mut self.enemy_bullets {
            bullet.physics();
        }

        //remove any enemies that died
        self.enemies.retain(|_, enemy| enemy.health > 0.0);
        //win state
        if self.enemies.len() == 0 {
            println!("\nyou win!\n");
            event::quit(ctx);
        }
        //lose state
        if self.player.health <= 0.0 {
            println!("\nyou lose!\n");
            event::quit(ctx);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        //render bullets
        for (_, bullet) in &mut self.bullets {
            self.spritebatch_bullet.add(
                DrawParam::default()
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .offset(Point2::new(0.5, 0.5))
                    .rotation(bullet.angle)
            );
            if SHOW_HITBOXES {
                for hitbox_visit in bullet.hitbox_tree.bfs_iter() {
                    let hitbox = hitbox_visit.data;
                    let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from(HITBOX_COLOR))?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
        }
        for (_, bullet) in &mut self.enemy_bullets {
            self.spritebatch_bullet.add(
                DrawParam::default()
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .offset(Point2::new(0.5, 0.5))
                    .rotation(bullet.angle)
            );
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
            let mut enemy_draw_param = DrawParam::default().dest(enemy.position);
            if enemy.flash_frames>0 {
                enemy_draw_param = enemy_draw_param.color(Color::new(3.0, 0.8, 0.8, 1.0));
            }
            self.spritebatch_enemy.add(enemy_draw_param);
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
        let mut player_draw_param = DrawParam::default().dest(self.player.position);
        if self.player.invincibility_frames>0 && self.player.invincibility_frames/(PLAYER_INVINCIBILITY/10)%2==0 {
            player_draw_param = player_draw_param.color(Color::new(1.0, 1.0, 1.0, 0.1));
        }
        self.spritebatch_player.add(player_draw_param);
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

        //player hud
        let hud_rect = Rect::new(10.0, 23.0*RESOLUTION.1/24.0-10.0, self.player.health/100.0*3.0*RESOLUTION.0/12.0, RESOLUTION.1/24.0);
        let hud_mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_rect, Color::new(1.0, 0.0, 0.0, 0.4))?;
        graphics::draw(ctx, &hud_mesh, DrawParam::default())?;

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
