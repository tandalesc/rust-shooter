use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, MeshBuilder, Rect, Text, TextFragment};
use ggez::graphics::spritebatch::{SpriteBatch};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::*;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_distr::StandardNormal;
use uuid::Uuid;
use std::collections::HashSet;
use std::collections::HashMap;

use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet, Star, GameObject};

const ENEMIES: (u8,u8) = (7, 3);
const ENEMY_SHOOT_CHANCE: usize = 10;

const PLAYER_EXP_PER_KILL: f32 = 40.0;
const PLAYER_INVINCIBILITY: u32 = 60;
const HITBOX_COLOR: (f32, f32, f32, f32) = (1.0, 0.1, 0.1, 0.4);

pub const FRICTION: f32 = 0.1;

pub const DISPLAY_RESOLUTION: (f32, f32) = (920.0, 690.0);
pub const INTERNAL_RESOLUTION: (f32, f32) = (640.0, 480.0);
pub const SCALED_RESOLUTION: (f32, f32) = (DISPLAY_RESOLUTION.0/INTERNAL_RESOLUTION.0, DISPLAY_RESOLUTION.1/INTERNAL_RESOLUTION.1);

const SHOW_FRAMERATE: bool = false;
const SHOW_HITBOXES: bool = false;

pub struct State {
    player: Player,
    bullets: HashMap<Uuid, Bullet>,
    enemy_bullets: HashMap<Uuid, Bullet>,
    enemies: HashMap<Uuid, Enemy>,
    stars: HashMap<Uuid, Star>,
    bullet_ids: HashSet<Uuid>,
    enemy_ids: HashSet<Uuid>,
    keys: HashSet<KeyCode>,
    rng: ThreadRng,
    status: Option<&'static str>,
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
            stars: HashMap::new(),
            bullet_ids: HashSet::new(),
            enemy_ids: HashSet::new(),
            keys: HashSet::with_capacity(6),
            rng: rand::thread_rng(),
            status: None,
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
                KeyCode::LShift => {
                    if self.player.bullet_spacing==0 {
                        self.player.cycle_weapons();
                        self.player.bullet_spacing += 50;
                    }
                }
                KeyCode::Space => {
                    if self.player.bullet_spacing==0 {
                        for bullet in self.player.shoot() { self.bullets.insert(Uuid::new_v4(), bullet); }
                        self.player.bullet_spacing = self.player.get_weapon().get_fire_rate();
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
                    enemy.health -= bullet.damage;
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
                self.player.health -= bullet.damage;
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
            let scaled_enemy_shoot_chance = ENEMY_SHOOT_CHANCE*num_enemies*num_enemies;
            if self.rng.gen_range(0, scaled_enemy_shoot_chance)==0 {
                let offset = Vector2::new(0.0, 20.0);
                //bullets go towards player
                let direction = self.player.position - enemy.position;
                //norm_squared is cheaper, and we want more sensivity anyway
                let dist = direction.norm();
                //add some noise so the enemies aren't always sniping us
                //but their accuracy goes up as they get closer
                let accuracy = 1.0/dist/num_enemies as f32;
                let normal_sample: f32 = self.rng.sample(StandardNormal);
                let noise = self.player.size/2.0 * normal_sample * (1.0-accuracy) * Vector2::new(1.0,1.0);
                let velocity = (direction + noise).normalize()*3.0;
                self.enemy_bullets.insert(Uuid::new_v4(), Bullet::new(enemy, velocity, Some(offset), 10.0));
            }
            //only check for collisions if player is not invincible
            if self.player.invincibility_frames==0 && enemy.collides_with(&self.player) {
                self.player.health -= 20.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            }
            if enemy.health <= 0.0 {
                //mark enemy for removal and add experience to player
                self.enemy_ids.insert(*enemy_id);
                self.player.experience += PLAYER_EXP_PER_KILL*(0.7_f32).powf(self.player.get_weapon().get_level() as f32);
            }
        }
        //remove any enemies that died
        for id in &self.enemy_ids { self.enemies.remove(id); };
        self.enemy_ids.clear();
        Ok(())
    }
    fn handle_background(&mut self, _ctx: &mut Context) -> GameResult<()> {
        //spawn stars occasionally
        if self.rng.gen_range(0.0, 1.0)<0.3 {
            //position is sampled uniformly across X-axis
            let random_position = Point2::new(self.rng.gen_range(0.0, DISPLAY_RESOLUTION.0), 0.0);
            //generate some random numbers to calculate size, velocity, and brightness with
            let normal_sample: f32 = self.rng.sample(StandardNormal);
            let normal_sample_2: f32 = self.rng.gen_range(0.0, 1.0);
            //these were experimentally determined biases and weights
            let random_size = 1.0 + 0.5 * normal_sample.abs();
            let velocity = Vector2::new(0.0, 0.3 + 0.3*normal_sample_2 + 0.3*normal_sample.abs());
            //insert our new star into the collection
            self.stars.insert(Uuid::new_v4(), Star::new(random_position, velocity, random_size, normal_sample_2));
        }
        //apply physics to existing stars
        for (_, star) in &mut self.stars { star.physics(); };
        //delete stars that are off-screen
        self.stars.retain(|_, star| !star.is_off_screen());
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
        //spawn stars
        self.handle_background(ctx)?;

        //handle player movement
        self.player.physics();
        //level up
        if self.player.experience >= 100.0 {
            self.player.experience = 0.0;
            self.player.get_weapon_mut().level_up();
        }

        //win states
        if self.enemies.len() == 0 {
            self.status = Some("you win!");
        }
        //lose states
        if self.player.health <= 0.0 {
            self.status = Some("game over");
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let mut background_meshbuilder = MeshBuilder::new();
        let mut hitbox_meshbuilder = MeshBuilder::new();
        let window_scaler = Vector2::new(SCALED_RESOLUTION.0, SCALED_RESOLUTION.1);
        let sprite_draw_params = DrawParam::default().scale(window_scaler);

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
                    hitbox_meshbuilder.rectangle(DrawMode::fill(), rect, Color::from(HITBOX_COLOR));
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
                    hitbox_meshbuilder.rectangle(DrawMode::fill(), rect, Color::from(HITBOX_COLOR));
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
                    hitbox_meshbuilder.rectangle(DrawMode::fill(), rect, Color::from(HITBOX_COLOR));
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
                hitbox_meshbuilder.rectangle(DrawMode::fill(), rect, Color::from(HITBOX_COLOR));
            }
        }

        //draw background layer
        for (_, star) in &mut self.stars {
            let dim = star.brightness*0.8;
            background_meshbuilder.circle(DrawMode::fill(), star.position, star.size, 1.0, Color::new(dim, dim, dim, 1.0));
        }


        graphics::set_default_filter(ctx, graphics::FilterMode::Nearest);
        //draw all accumulated meshes
        if let Ok(mesh) = background_meshbuilder.build(ctx) {
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }
        if let Ok(mesh) = hitbox_meshbuilder.build(ctx) {
            graphics::draw(ctx, &mesh, sprite_draw_params)?;
        }

        //draw sprites, clear spritebatches
        graphics::draw(ctx, &self.spritebatch_bullet, sprite_draw_params)?;
        graphics::draw(ctx, &self.spritebatch_enemy, sprite_draw_params)?;
        graphics::draw(ctx, &self.spritebatch_player, sprite_draw_params)?;
        self.spritebatch_bullet.clear();
        self.spritebatch_enemy.clear();
        self.spritebatch_player.clear();

        //player hud
        //health
        //divide resolution by 12.0 or 24.0 and use as units with 10px as padding on edges
        let hud_health_position = Point2::new(10.0, 23.0*DISPLAY_RESOLUTION.1/24.0-10.0);
        let hud_health_outline_rect = Rect::new(hud_health_position.x, hud_health_position.y, 3.0*DISPLAY_RESOLUTION.0/12.0, DISPLAY_RESOLUTION.1/24.0);
        let hud_health_rect = Rect::new(10.0, 23.0*DISPLAY_RESOLUTION.1/24.0-10.0, self.player.health/100.0*3.0*DISPLAY_RESOLUTION.0/12.0, DISPLAY_RESOLUTION.1/24.0);
        let hud_health_outline = Mesh::new_rectangle(ctx, DrawMode::stroke(1.2), hud_health_outline_rect, Color::new(1.0, 0.0, 0.0, 0.8))?;
        let hud_health_filled = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_health_rect, Color::new(1.0, 0.0, 0.0, 0.3))?;
        graphics::draw(ctx, &hud_health_outline, DrawParam::default())?;
        graphics::draw(ctx, &hud_health_filled, DrawParam::default())?;
        //experience
        //divide resolution by 12.0 or 24.0 and use as units with 10px as padding on edges
        let hud_exp_position = Point2::new(9.0*DISPLAY_RESOLUTION.0/12.0-10.0, 23.0*DISPLAY_RESOLUTION.1/24.0-10.0);
        let hud_exp_outline_rect = Rect::new(hud_exp_position.x, hud_exp_position.y, 3.0*DISPLAY_RESOLUTION.0/12.0, DISPLAY_RESOLUTION.1/24.0);
        let hud_exp_rect = Rect::new(9.0*DISPLAY_RESOLUTION.0/12.0-10.0, 23.0*DISPLAY_RESOLUTION.1/24.0-10.0, self.player.experience/100.0*3.0*DISPLAY_RESOLUTION.0/12.0, DISPLAY_RESOLUTION.1/24.0);
        let hud_exp_outline = Mesh::new_rectangle(ctx, DrawMode::stroke(1.2), hud_exp_outline_rect, Color::new(0.0, 1.0, 0.0, 0.8))?;
        let hud_exp_filled = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_exp_rect, Color::new(0.0, 1.0, 0.0, 0.3))?;
        graphics::draw(ctx, &hud_exp_outline, DrawParam::default())?;
        graphics::draw(ctx, &hud_exp_filled, DrawParam::default())?;

        //health information
        let health_text = Text::new(TextFragment::new(format!("health: {}",self.player.health)));
        graphics::draw(ctx, &health_text, DrawParam::default().dest(Point2::new(hud_health_position.x+10.0, hud_health_position.y+10.0)))?;
        //weapon information
        let weapon_text = Text::new(TextFragment::new(self.player.get_weapon().get_info()));
        graphics::draw(ctx, &weapon_text, DrawParam::default().dest(Point2::new(hud_exp_position.x+10.0, hud_exp_position.y+10.0)))?;
        //status text
        if let Some(status_text) = self.status {
            let text = Text::new(TextFragment::new(status_text));
            graphics::draw(ctx, &text, DrawParam::default().dest(Point2::new(DISPLAY_RESOLUTION.0/2.0-50.0, DISPLAY_RESOLUTION.1/2.0)))?;
        }
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
