use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, MeshBuilder, Rect, Text, TextFragment};
use ggez::graphics::spritebatch::{SpriteBatch};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::*;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_distr::StandardNormal;
use std::collections::HashSet;
use std::io::Read;
use std::str;

use crate::spritesheet::SpriteSheetData;
use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet, Star, GameObject};

const ENEMIES: (u8,u8) = (7, 3);
const ENEMY_SHOOT_CHANCE: usize = 10;

const PLAYER_EXP_PER_KILL: f32 = 40.0;
const PLAYER_INVINCIBILITY: u32 = 60;
const HITBOX_COLOR: (f32, f32, f32, f32) = (1.0, 0.1, 0.1, 0.4);

pub const FRICTION: f32 = 0.1;

pub const DISPLAY_RESOLUTION: (f32, f32) = (920.0, 690.0);
pub const INTERNAL_RESOLUTION: (f32, f32) = (920.0, 690.0);

const SHOW_FRAMERATE: bool = false;
const SHOW_HITBOXES: bool = false;

pub struct State {
    player: Player,
    bullets: Vec<Bullet>,
    enemy_bullets: Vec<Bullet>,
    enemies: Vec<Enemy>,
    stars: Vec<Star>,
    keys: HashSet<KeyCode>,
    rng: ThreadRng,
    status: Option<&'static str>,
    spritesheet_data: SpriteSheetData,
    spritebatch_spritesheet: SpriteBatch
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        let mut buffer = Vec::new();
        //load sprite sheet information from texturepacker json using serde
        let mut spritesheet_data_file = filesystem::open(ctx, "/spaceship_sprites.json")?;
        spritesheet_data_file.read_to_end(&mut buffer)?;
        let spritesheet_data: SpriteSheetData = serde_json::from_str(str::from_utf8(&buffer).unwrap()).unwrap();
        //print out info about the first sprite as a test
        //println!("{:?}", sprite_sheet_data.frames.get("Spaceships/1").unwrap());
        let mut state = State {
            player: Player::new(),
            bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            enemies: Vec::new(),
            stars: Vec::new(),
            keys: HashSet::with_capacity(6),
            rng: rand::thread_rng(),
            status: None,
            spritesheet_data: spritesheet_data,
            spritebatch_spritesheet: SpriteBatch::new(Image::new(ctx, "/spaceship_sprites.png").unwrap())
        };
        //generate a grid of enemies
        //todo: refactor into it's own method so we can generate enemies on the fly
        for x in 0..ENEMIES.0 {
            for y in 0..ENEMIES.1 {
                state.enemies.push(
                    Enemy::new(
                        Point2::new(80.0, 50.0) +
                        (x as f32)*Vector2::new(110.0, 0.0) +
                        (y as f32)*Vector2::new(0.0, 100.0)
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
                        for bullet in self.player.shoot() {
                            self.bullets.push(bullet);
                        }
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
        for bullet in &mut self.bullets {
            //apply physics to player bullets
            bullet.physics();
            for enemy in &mut self.enemies {
                if bullet.collides_with(enemy) {
                    //mark bullet for deletion
                    bullet.alive = false;
                    //do damage
                    enemy.health -= bullet.damage;
                    enemy.flash_frames = 5;
                }
            }
        }
        self.bullets.retain(|bullet| bullet.alive && !bullet.is_off_screen());

        //check enemy bullets and player for collisions
        for bullet in &mut self.enemy_bullets {
            //apply physics to enemy bullets
            bullet.physics();
            //only check for collisions if player is not invincible
            if self.player.invincibility_frames==0 && bullet.collides_with(&self.player) {
                //mark bullet for deletion
                 bullet.alive = false;
                //do damage
                self.player.health -= bullet.damage;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            }
        }
        self.enemy_bullets.retain(|bullet| bullet.alive && !bullet.is_off_screen());

        Ok(())
    }
    fn handle_enemies(&mut self, _ctx: &mut Context) -> GameResult<()> {
        let num_enemies = self.enemies.len();
        for enemy in &mut self.enemies {
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
                self.enemy_bullets.push(Bullet::new(enemy, velocity, Some(offset), 10.0));
            }
            //only check for collisions if player is not invincible
            if self.player.invincibility_frames==0 && enemy.collides_with(&self.player) {
                self.player.health -= 20.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            }
            if enemy.health <= 0.0 {
                //mark enemy for removal and add experience to player
                enemy.alive = false;
                self.player.experience += PLAYER_EXP_PER_KILL*(0.7_f32).powf(self.player.get_weapon().get_level() as f32);
            }
        }
        //remove any enemies that died
        self.enemies.retain(|enemy| enemy.alive);
        Ok(())
    }
    fn handle_background(&mut self, _ctx: &mut Context) -> GameResult<()> {
        //spawn stars occasionally
        if self.rng.gen_range(0.0, 1.0)<0.03 {
            //position is sampled uniformly across X-axis
            let random_position = Point2::new(self.rng.gen_range(0.0, DISPLAY_RESOLUTION.0), 0.0);
            //generate some random numbers to calculate size, velocity, and brightness with
            let normal_sample: f32 = self.rng.sample(StandardNormal);
            let normal_sample_2: f32 = self.rng.gen_range(0.0, 1.0);
            //these were experimentally determined biases and weights
            let random_size = 1.0 + 0.5 * normal_sample.abs();
            let velocity = Vector2::new(0.0, 0.3 + 0.3*normal_sample_2 + 0.3*normal_sample.abs());
            //insert our new star into the collection
            self.stars.push(Star::new(random_position, velocity, random_size, normal_sample_2));
        }
        //apply physics to existing stars
        for star in &mut self.stars { star.physics(); };
        //delete stars that are off-screen
        self.stars.retain(|star| !star.is_off_screen());
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        //only print out periodically, one iteration per one frame, assuming 1fps
        while timer::check_update_time(ctx, 1) {
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

        let spritesheet_rect = self.spritesheet_data.meta.size.to_rect_f32();
        let halfway_point = Point2::new(0.5, 0.5);
        let sprite_scale = Vector2::new(0.30, 0.30);
        let spritesheet_draw_params = DrawParam::default()
            .offset(halfway_point)
            .scale(sprite_scale);

        //render player bullets
        let bullet_spr_info = self.spritesheet_data.frames.get("energy_blast").unwrap().frame.to_rect_f32();
        let adjusted_bullet_spr_info = Rect::fraction(
            bullet_spr_info.x,
            bullet_spr_info.y,
            bullet_spr_info.w,
            bullet_spr_info.h,
            &spritesheet_rect
        );
        let player_bullet_color = Color::new(0.4,0.4,1.0,1.0);
        for bullet in &mut self.bullets {
            self.spritebatch_spritesheet.add(
                spritesheet_draw_params
                    .src(adjusted_bullet_spr_info)
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .color(player_bullet_color)
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
        let enemy_bullet_color = Color::new(0.8,0.1,0.1,1.0);
        for bullet in &mut self.enemy_bullets {
            self.spritebatch_spritesheet.add(
                spritesheet_draw_params
                    .src(adjusted_bullet_spr_info)
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .color(enemy_bullet_color)
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
        let enemy_spr_info = self.spritesheet_data.frames.get("spaceship_1").unwrap().frame.to_rect_f32();
        let adjusted_enemy_sprite_coor = Rect::fraction(
            enemy_spr_info.x,
            enemy_spr_info.y,
            enemy_spr_info.w,
            enemy_spr_info.h,
            &spritesheet_rect
        );
        for enemy in &mut self.enemies {
            let mut enemy_draw_param = spritesheet_draw_params
                .src(adjusted_enemy_sprite_coor)
                .rotation(std::f32::consts::PI)
                .dest(enemy.position + Vector2::new(enemy.size/2.0, enemy.size/2.0));
            if enemy.flash_frames>0 {
                //flash enemies when hit
                enemy_draw_param = enemy_draw_param.color(Color::new(3.0, 0.8, 0.8, 1.0));
            }
            self.spritebatch_spritesheet.add(enemy_draw_param);
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
        let player_spaceship = format!("spaceship_{}", self.player.current_weapon_idx+2);
        let player_spr_info = self.spritesheet_data.frames.get(&player_spaceship).unwrap().frame.to_rect_f32();
        let adjusted_enemy_sprite_coor = Rect::fraction(
            player_spr_info.x,
            player_spr_info.y,
            player_spr_info.w,
            player_spr_info.h,
            &spritesheet_rect
        );
        let mut player_draw_param = spritesheet_draw_params
            .src(adjusted_enemy_sprite_coor)
            .dest(self.player.position + Vector2::new(self.player.size/2.0, self.player.size/2.0));
        if self.player.invincibility_frames>0 && self.player.invincibility_frames/(PLAYER_INVINCIBILITY/10)%2==0 {
            //flash player when invincible
            player_draw_param = player_draw_param.color(Color::new(1.0, 1.0, 1.0, 0.1));
        }
        self.spritebatch_spritesheet.add(player_draw_param);
        //draw hitboxes for debugging
        if SHOW_HITBOXES {
            for hitbox_visit in self.player.hitbox_tree.bfs_iter() {
                let hitbox = hitbox_visit.data;
                let rect = Rect::new(hitbox.point.x, hitbox.point.y, hitbox.size.x, hitbox.size.y);
                hitbox_meshbuilder.rectangle(DrawMode::fill(), rect, Color::from(HITBOX_COLOR));
            }
        }

        //build background layer
        for star in &mut self.stars {
            let dim = star.brightness*0.8;
            background_meshbuilder.circle(DrawMode::fill(), star.position, star.size, 1.0, Color::new(dim, dim, dim, 1.0));
        }

        //draw all accumulated meshes
        if let Ok(mesh) = background_meshbuilder.build(ctx) {
            graphics::draw(ctx, &mesh, DrawParam::default())?;
        }
        //draw sprites, clear spritebatches
        graphics::draw(ctx, &self.spritebatch_spritesheet, DrawParam::default())?;
        self.spritebatch_spritesheet.clear();
        //draw hitboxes, if enabled
        if SHOW_HITBOXES {
            if let Ok(mesh) = hitbox_meshbuilder.build(ctx) {
                graphics::draw(ctx, &mesh, DrawParam::default())?;
            }
        }

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
