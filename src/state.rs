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

use crate::spritesheet::{SpriteAnimationComponent, SpriteAnimation, SpriteSheetData, SpriteAnimationSystem, SpriteAnimationRegistry, SpriteObject};
use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet, BulletType, Star, GameObject, Explosion};
use crate::weapon::{Weapon};

const ENEMIES: (u8,u8) = (7, 3);
const ENEMY_SHOOT_CHANCE: usize = 10;

const NUM_STARS: usize = 300;

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
    explosions: Vec<Explosion>,
    stars: Vec<Star>,
    keys: HashSet<KeyCode>,
    rng: ThreadRng,
    stage: usize,
    no_attack_timer: usize,
    status: Option<&'static str>,
    animation_registry: SpriteAnimationRegistry,
    animation_system: SpriteAnimationSystem,
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
        let mut animation_registry = SpriteAnimationRegistry::new();
        animation_registry.add_anim(
            "explosion".to_string(),
            SpriteAnimation {
                frames: vec![
                    "Explosion01_Frame_01".to_string(),
                    "Explosion01_Frame_02".to_string(),
                    "Explosion01_Frame_03".to_string(),
                    "Explosion01_Frame_04".to_string(),
                    "Explosion01_Frame_05".to_string(),
                    "Explosion01_Frame_06".to_string(),
                    "Explosion01_Frame_07".to_string(),
                    "Explosion01_Frame_08".to_string(),
                    "Explosion01_Frame_09".to_string()
                ],
                time_per_frame: 1000.0/24.0,
                loop_anim: false
            }
        );
        animation_registry.add_anim(
            "player_turn".to_string(),
            SpriteAnimation {
                frames: vec![
                    "PlayerBlue_Frame_01".to_string(),
                    "PlayerBlue_Frame_02".to_string(),
                    "PlayerBlue_Frame_03".to_string()
                ],
                time_per_frame: 1000.0/12.0,
                loop_anim: false
            }
        );
        let state = State {
            player: Player::new(),
            bullets: Vec::new(),
            enemy_bullets: Vec::new(),
            enemies: Vec::new(),
            explosions: Vec::new(),
            stars: Vec::new(),
            keys: HashSet::with_capacity(6),
            rng: rand::thread_rng(),
            stage: 0,
            no_attack_timer: 0,
            status: None,
            animation_registry: animation_registry,
            animation_system: SpriteAnimationSystem::new(),
            spritesheet_data: spritesheet_data,
            spritebatch_spritesheet: SpriteBatch::new(Image::new(ctx, "/spaceship_sprites.png").unwrap())
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
                KeyCode::LShift => {
                    if self.player.alive && self.player.bullet_spacing==0 {
                        self.player.cycle_weapons();
                        self.player.bullet_spacing += 50;
                    }
                }
                KeyCode::Space => {
                    if self.player.alive && self.player.bullet_spacing==0 && self.no_attack_timer==0 {
                        for bullet in self.player.shoot() {
                            self.bullets.push(bullet);
                        }
                        self.player.bullet_spacing = self.player.get_weapon().unwrap().get_fire_rate();
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
                if self.no_attack_timer==0 && bullet.collides_with(enemy) {
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
            if self.player.alive && self.player.invincibility_frames==0 && self.no_attack_timer==0 && bullet.collides_with(&self.player) {
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
            let scaled_enemy_shoot_chance = ENEMY_SHOOT_CHANCE*num_enemies*num_enemies/self.stage;
            if self.player.alive && self.no_attack_timer==0 && self.rng.gen_range(0, scaled_enemy_shoot_chance)==0 {
                let offset = Vector2::new(0.0, 20.0);
                //bullets go towards player
                let direction = self.player.position - enemy.position;
                let dist = direction.norm();
                //add some noise so the enemies aren't always sniping us
                //but their accuracy goes up as they get closer
                let accuracy = 1.0/dist/num_enemies as f32;
                let normal_sample: f32 = self.rng.sample(StandardNormal);
                let noise = self.player.size/2.0 * normal_sample * (1.0-accuracy) * Vector2::new(1.0,1.0);
                let velocity = (direction + noise).normalize()*3.0;
                self.enemy_bullets.push(Bullet::new(enemy, velocity, Some(offset), 10.0, BulletType::Proton));
            }
            //only check for collisions if player is not invincible
            if self.player.invincibility_frames==0 && self.no_attack_timer==0 && enemy.collides_with(&self.player) {
                self.player.health -= 20.0;
                self.player.invincibility_frames = PLAYER_INVINCIBILITY;
            }
            if enemy.health <= 0.0 {
                //mark enemy for removal and add experience to player
                enemy.alive = false;
                self.player.experience += PLAYER_EXP_PER_KILL*(0.7_f32).powf(self.player.get_weapon().unwrap().get_level() as f32);
            }
        }
        for enemy in &self.enemies {
            //create an explosion
            if !enemy.alive {
                self.explosions.push(Explosion::new(enemy.position, enemy.velocity, 64.,
                        &mut self.animation_system, &self.animation_registry
                ));
            }
        }
        //remove any enemies that died
        self.enemies.retain(|enemy| enemy.alive);
        Ok(())
    }
    fn handle_background(&mut self, _ctx: &mut Context) -> GameResult<()> {
        //spawn stars occasionally until we reach our max number of stars
        if self.stars.len() < NUM_STARS && self.rng.gen_range(0.0, 1.0)<0.3 {
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
        for star in &mut self.stars {
            //wrap stars around the screen
            if star.position.y > DISPLAY_RESOLUTION.1 {
                star.position.y = 0.0;
            }
            star.physics();
        };
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
        if self.no_attack_timer>0 {
            while timer::check_update_time(ctx, 60) && self.no_attack_timer>0 {
                self.no_attack_timer -= 1;
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

        //keep explosions that aren't finished
        for exp in &mut self.explosions {
            if exp.poll_animation_finished(&self.animation_system) {
                exp.finished = true;
            }
        }
        self.explosions.retain(|exp| !exp.finished);

        //update animations
        let timer_delta = timer::delta(ctx).as_millis() as f32;
        self.animation_system.time_tick(timer_delta);

        //handle player movement
        self.player.physics();
        //level up
        if self.player.experience >= 100.0 {
            self.player.experience = 0.0;
            self.player.get_weapon_mut().unwrap().level_up();
        }

        //win states
        if self.player.alive && self.enemies.len()==0 {
            //increment stage counter
            self.stage += 1;
            self.no_attack_timer = 200;
            //restore up to 25% of player health
            if self.player.health < 25. {
                self.player.health = 25.;
            }
            //generate a grid of enemies
            for x in 0..ENEMIES.0 {
                for y in 0..ENEMIES.1 {
                    self.enemies.push(
                        Enemy::new(
                            Point2::new(80.0, 50.0) +
                            (x as f32)*Vector2::new(110.0, 0.0) +
                            (y as f32)*Vector2::new(0.0, 100.0)
                        )
                    );
                }
            }
        }
        //lose states
        if self.player.alive && self.player.health <= 0.0 {
            self.status = Some("game over");
            self.player.alive = false;
            self.player.health = 0.;
            self.explosions.push(Explosion::new(self.player.position, Vector2::new(0.,0.), 64.,
                    &mut self.animation_system, &self.animation_registry
            ));
        }
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);
        let mut background_meshbuilder = MeshBuilder::new();
        let mut hitbox_meshbuilder = MeshBuilder::new();

        let halfway_point = Point2::new(0.5, 0.5);
        let sprite_scale = Vector2::new(1., 1.);
        let spritesheet_draw_params = DrawParam::default()
            .offset(halfway_point)
            .scale(sprite_scale);

        //render player bullets
        for bullet in &mut self.bullets {
            let bullet_spr_info = bullet.get_fractional_frame(&self.animation_system, &self.spritesheet_data).unwrap();
            self.spritebatch_spritesheet.add(
                spritesheet_draw_params
                    .src(bullet_spr_info)
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .rotation(bullet.angle)
                    //.color(player_bullet_color)
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
        for bullet in &mut self.enemy_bullets {
            let bullet_spr_info = bullet.get_fractional_frame(&self.animation_system, &self.spritesheet_data).unwrap();
            self.spritebatch_spritesheet.add(
                spritesheet_draw_params
                    .src(bullet_spr_info)
                    .dest(bullet.position + Vector2::new(bullet.size/2.0, bullet.size/2.0))
                    .rotation(bullet.angle)
                    //.color(enemy_bullet_color)
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
        for enemy in &mut self.enemies {
            let enemy_spr_info = enemy.get_fractional_frame(&self.animation_system, &self.spritesheet_data).unwrap();
            let mut enemy_draw_param = spritesheet_draw_params
                .src(enemy_spr_info)
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

        //render explosions
        for exp in &mut self.explosions {
            let exp_sprite_info = exp.get_fractional_frame(&self.animation_system, &self.spritesheet_data).unwrap();
            let mut exp_draw_params = spritesheet_draw_params
                .src(exp_sprite_info)
                .dest(exp.position + Vector2::new(exp.size/2.0, exp.size/2.0));

            self.spritebatch_spritesheet.add(exp_draw_params);
        }

        //render player
        if self.player.alive {
            let player_spr_info = self.player.get_fractional_frame(&self.animation_system, &self.spritesheet_data).unwrap();
            let mut player_draw_param = spritesheet_draw_params
                .src(player_spr_info)
                .dest(self.player.position + Vector2::new(self.player.size/2.0, self.player.size/2.0));
            let flash_player_when_invincible = self.player.invincibility_frames>0 && timer::ticks(ctx)/(PLAYER_INVINCIBILITY as usize/10)%2==0;
            let flash_player_no_attack_allowed = self.no_attack_timer>0 && timer::ticks(ctx)/(PLAYER_INVINCIBILITY as usize/10)%2==0;
            if flash_player_when_invincible || flash_player_no_attack_allowed {
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
        let hud_health_outline = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), hud_health_outline_rect, Color::new(1.0, 0.0, 0.0, 0.8))?;
        let hud_health_filled = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_health_rect, Color::new(1.0, 0.0, 0.0, 0.3))?;
        graphics::draw(ctx, &hud_health_outline, DrawParam::default())?;
        graphics::draw(ctx, &hud_health_filled, DrawParam::default())?;
        //experience
        //divide resolution by 12.0 or 24.0 and use as units with 10px as padding on edges
        let hud_exp_position = Point2::new(9.0*DISPLAY_RESOLUTION.0/12.0-10.0, 23.0*DISPLAY_RESOLUTION.1/24.0-10.0);
        let hud_exp_outline_rect = Rect::new(hud_exp_position.x, hud_exp_position.y, 3.0*DISPLAY_RESOLUTION.0/12.0, DISPLAY_RESOLUTION.1/24.0);
        let hud_exp_rect = Rect::new(9.0*DISPLAY_RESOLUTION.0/12.0-10.0, 23.0*DISPLAY_RESOLUTION.1/24.0-10.0, self.player.experience/100.0*3.0*DISPLAY_RESOLUTION.0/12.0, DISPLAY_RESOLUTION.1/24.0);
        let hud_exp_outline = Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), hud_exp_outline_rect, Color::new(0.0, 1.0, 0.0, 0.8))?;
        let hud_exp_filled = Mesh::new_rectangle(ctx, DrawMode::fill(), hud_exp_rect, Color::new(0.0, 1.0, 0.0, 0.3))?;
        graphics::draw(ctx, &hud_exp_outline, DrawParam::default())?;
        graphics::draw(ctx, &hud_exp_filled, DrawParam::default())?;

        //health information
        let health_text = Text::new(TextFragment::new(format!("health: {}/100",self.player.health)));
        graphics::draw(ctx, &health_text, DrawParam::default().dest(Point2::new(hud_health_position.x+10.0, hud_health_position.y+10.0)))?;
        //weapon information
        let weapon_text = Text::new(TextFragment::new(self.player.get_weapon().unwrap().get_info()));
        graphics::draw(ctx, &weapon_text, DrawParam::default().dest(Point2::new(hud_exp_position.x+10.0, hud_exp_position.y+10.0)))?;
        //current stage
        let stage_text_display = if self.no_attack_timer>0 {
            format!("timer: {}", self.no_attack_timer)
        } else {
            format!("stage: {}",self.stage)
        };
        let stage_text = Text::new(TextFragment::new(stage_text_display));
        graphics::draw(ctx, &stage_text, DrawParam::default().dest(Point2::new(hud_health_position.x+10.0, 10.0)))?;
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
