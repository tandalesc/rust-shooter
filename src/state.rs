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
const BULLET_SPACING_X: f32 = 5.0;
const BULLET_SPACING_Y: f32 = 0.7;
//bullet-spacing in time axis (num frames)
const BULLET_SPACING_T: i16 = 10;
const BULLET_OFFSET: f32 = -3.0;
const BULLET_SPREAD: i16 = 2;
const BULLET_ANGLE: f32 = 0.3;
const BULLET_SPEED: f32 = 5.0;
pub const BULLET_SIZE: f32 = 10.0;

const ENEMIES: (u8,u8) = (7, 3);
const ENEMY_SHOOT_CHANCE: usize = 50;

const PLAYER_INVINCIBILITY: i16 = 60;
const HITBOX_COLOR: (f32, f32, f32, f32) = (1.0, 0.1, 0.1, 0.4);

pub const FRICTION: f32 = 0.1;

pub const RESOLUTION: (f32, f32) = (640.0, 480.0);
//sub-divides screen into 10x10 grid for the purposes of speeding up collision detection
pub const GRID_RESOLUTION: (f32, f32) = (10.0, 10.0);

const SHOW_FRAMERATE: bool = false;
const SHOW_HITBOXES: bool = false;

pub struct State {
    player: Player,
    bullets: HashMap<Uuid, Bullet>,
    enemy_bullets: HashMap<Uuid, Bullet>,
    enemies: HashMap<Uuid, Enemy>,
    bullet_ids: HashSet<Uuid>,
    enemy_ids: HashSet<Uuid>,
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
            enemies: HashMap::new(),
            bullet_ids: HashSet::new(),
            enemy_ids: HashSet::new(),
            keys: HashSet::with_capacity(6),
            rng: rand::thread_rng(),
            spritebatch_player: SpriteBatch::new(Image::new(ctx, "/player.png").unwrap()),
            spritebatch_bullet: SpriteBatch::new(Image::new(ctx, "/bullet.png").unwrap()),
            spritebatch_enemy: SpriteBatch::new(Image::new(ctx, "/enemy.png").unwrap())
        };
        //generate a grid of enemies
        //todo: refactor into it's own method so we can generate enemies on the fly
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
                    //todo: refactor into different WeaponTypes and allow switching
                    if self.player.bullet_spacing==0 {
                        //scale number of bullets with weapon level
                        for _bullet_num in -self.player.weapon_level..(self.player.weapon_level+1) {
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
        for (bullet_id, bullet) in &mut self.bullets {
            //apply physics to player bullets
            bullet.physics();
            for (_, enemy) in &mut self.enemies {
                if bullet.collides_with(enemy) {
                    //mark bullet for deletion
                    self.bullet_ids.insert(*bullet_id);
                    //do damage
                    enemy.health -= BULLET_DAMAGE;
                    enemy.flash_frames = 5;
                } else if bullet.is_off_screen() {
                    //mark bullet for deletion
                    self.bullet_ids.insert(*bullet_id);
                }
            }
        }
        //check enemy bullets and player for collisions
        for (bullet_id, bullet) in &mut self.enemy_bullets {
            //apply physics to enemy bullets
            bullet.physics();
            //only check for collisions if player is not invincible
            if self.player.invincibility_frames==0 && bullet.collides_with(&self.player) {
                //mark bullet for deletion
                self.bullet_ids.insert(*bullet_id);
                //do damage
                self.player.health -= 10.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            } else if bullet.is_off_screen() {
                //mark bullet for deletion
                self.bullet_ids.insert(*bullet_id);
            }
        }
        //remove any bullets that were marked for deletion
        for id in &self.bullet_ids {
            self.bullets.remove(id);
            self.enemy_bullets.remove(id);
        }
        self.bullet_ids.clear();
        Ok(())
    }
    fn handle_enemies(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let num_enemies = self.enemies.len();
        for (enemy_id, enemy) in &mut self.enemies {
            enemy.physics();
            //scale shooting chance with number of enemies
            //more enemies = each one shoots less frequently
            if self.rng.gen_range(0, ENEMY_SHOOT_CHANCE*num_enemies)==0 {
                let offset = Vector2::new(0.0, 20.0);
                //bullets go towards player
                let velocity = (self.player.position - enemy.position).normalize()*BULLET_SPEED;
                self.enemy_bullets.insert(Uuid::new_v4(), Bullet::new(enemy, velocity, Some(offset)));
            }
            //only check for collisions if player is not invincible
            if self.player.invincibility_frames==0 && enemy.collides_with(&self.player) {
                self.player.health -= 20.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            }
            if enemy.health <= 0.0 {
                //mark enemy for removal and add experience to player
                self.enemy_ids.insert(*enemy_id);
                self.player.experience += 50.0/((self.player.weapon_level+1) as f32);
            }
        }
        //remove any enemies that died
        for id in &self.enemy_ids { self.enemies.remove(id); };
        self.enemy_ids.clear();
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        //only print out periodically
        if timer::ticks(ctx) % 50 == 0 {
            if SHOW_FRAMERATE {
                println!("Average FPS: {}, #Bullets: {}", timer::fps(ctx), self.bullets.len()+self.enemy_bullets.len());
            }
        }

        //better handle key presses and releases
        self.handle_keys(ctx)?;
        //apply game logic to bullets (and enemies/players who are hit)
        self.handle_bullets(ctx)?;
        //apply game logic to enemies
        self.handle_enemies(ctx)?;

        //handle player movement
        self.player.physics();
        //level up
        if self.player.experience >= 100.0 {
            println!("\nlevel up!\n");
            self.player.experience = 0.0;
            self.player.weapon_level += 1;
        }

        //win states
        if self.enemies.len() == 0 {
            println!("\nyou win!\n");
            event::quit(ctx);
        }
        //lose states
        if self.player.health <= 0.0 {
            println!("\nyou lose!\n");
            event::quit(ctx);
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        //render player bullets
        for (_, bullet) in &mut self.bullets {
            self.spritebatch_bullet.add(
                //rotate bullets in the direction of movement
                DrawParam::default()
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .offset(Point2::new(0.5, 0.5))
                    .rotation(bullet.angle)
            );
            //draw hitboxes for debugging
            if SHOW_HITBOXES {
                for hitbox_visit in bullet.hitbox_tree.bfs_iter() {
                    let hitbox = hitbox_visit.data;
                    let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from(HITBOX_COLOR))?;
                    graphics::draw(ctx, &mesh, DrawParam::default())?;
                }
            }
        }
        //render enemy bullets
        for (_, bullet) in &mut self.enemy_bullets {
            self.spritebatch_bullet.add(
                //rotate bullets in the direction of movement
                DrawParam::default()
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .offset(Point2::new(0.5, 0.5))
                    .rotation(bullet.angle)
            );
            //draw hitboxes for debugging
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
                //flash enemies when hit
                enemy_draw_param = enemy_draw_param.color(Color::new(3.0, 0.8, 0.8, 1.0));
            }
            self.spritebatch_enemy.add(enemy_draw_param);
            //draw hitboxes for debugging
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
            //flash player when invincible
            player_draw_param = player_draw_param.color(Color::new(1.0, 1.0, 1.0, 0.1));
        }
        self.spritebatch_player.add(player_draw_param);
        //draw hitboxes for debugging
        if SHOW_HITBOXES {
            for hitbox_visit in self.player.hitbox_tree.bfs_iter() {
                let hitbox = hitbox_visit.data;
                let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, Color::from(HITBOX_COLOR))?;
                graphics::draw(ctx, &mesh, DrawParam::default())?;
            }
        }

        //draw sprites, clear spritebatches
        graphics::draw(ctx, &self.spritebatch_bullet, DrawParam::default())?;
        graphics::draw(ctx, &self.spritebatch_enemy, DrawParam::default())?;
        graphics::draw(ctx, &self.spritebatch_player, DrawParam::default())?;
        self.spritebatch_bullet.clear();
        self.spritebatch_enemy.clear();
        self.spritebatch_player.clear();

        //player hud
        //health
        //divide resolution by 12.0 or 24.0 and use as units with 10px as padding on edges
        let hud_health_outline_rect = Rect::new(10.0, 23.0*RESOLUTION.1/24.0-10.0, 3.0*RESOLUTION.0/12.0, RESOLUTION.1/24.0);
        let hud_health_rect = Rect::new(10.0, 23.0*RESOLUTION.1/24.0-10.0, self.player.health/100.0*3.0*RESOLUTION.0/12.0, RESOLUTION.1/24.0);
        let hud_health_outline = Mesh::new_rectangle(ctx, DrawMode::stroke(1.2), hud_health_outline_rect, Color::new(1.0, 0.0, 0.0, 0.8))?;
        let hud_health_filled = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_health_rect, Color::new(1.0, 0.0, 0.0, 0.3))?;
        graphics::draw(ctx, &hud_health_outline, DrawParam::default())?;
        graphics::draw(ctx, &hud_health_filled, DrawParam::default())?;
        //experience
        //divide resolution by 12.0 or 24.0 and use as units with 10px as padding on edges
        let hud_exp_outline_rect = Rect::new(9.0*RESOLUTION.0/12.0-10.0, 23.0*RESOLUTION.1/24.0-10.0, 3.0*RESOLUTION.0/12.0, RESOLUTION.1/24.0);
        let hud_exp_rect = Rect::new(9.0*RESOLUTION.0/12.0-10.0, 23.0*RESOLUTION.1/24.0-10.0, self.player.experience/100.0*3.0*RESOLUTION.0/12.0, RESOLUTION.1/24.0);
        let hud_exp_outline = Mesh::new_rectangle(ctx, DrawMode::stroke(1.2), hud_exp_outline_rect, Color::new(0.0, 1.0, 0.0, 0.8))?;
        let hud_exp_filled = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_exp_rect, Color::new(0.0, 1.0, 0.0, 0.3))?;
        graphics::draw(ctx, &hud_exp_outline, DrawParam::default())?;
        graphics::draw(ctx, &hud_exp_filled, DrawParam::default())?;

        //end
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
