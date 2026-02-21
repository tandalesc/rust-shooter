use ggez::graphics::{Color, DrawMode, DrawParam, Image, Mesh, Rect, Text, TextFragment, Canvas, InstanceArray};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::event::EventHandler;
use ggez::glam::Vec2;
use ggez::*;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_distr::StandardNormal;
use std::collections::HashSet;
use std::io::Read;
use std::str;

use crate::config::*;
use crate::spritesheet::{SpriteAnimation, SpriteSheetData, SpriteAnimationSystem, SpriteAnimationRegistry, SpriteObject};
use crate::shooter::{Player, Enemy, Bullet, BulletType, Star, GameObject, Explosion};

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
    tick_count: usize,
    status: Option<&'static str>,
    animation_registry: SpriteAnimationRegistry,
    animation_system: SpriteAnimationSystem,
    spritesheet_data: SpriteSheetData,
    spritesheet_instances: InstanceArray,
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        let mut buffer = Vec::new();
        let mut spritesheet_data_file = ctx.fs.open("/spaceship_sprites.json")?;
        spritesheet_data_file.read_to_end(&mut buffer)?;
        let spritesheet_data: SpriteSheetData =
            serde_json::from_str(str::from_utf8(&buffer).unwrap()).unwrap();

        let mut animation_registry = SpriteAnimationRegistry::new();
        animation_registry.add_anim(
            "explosion".to_string(),
            SpriteAnimation::new(
                (1..=9).map(|i| format!("Explosion01_Frame_{i:02}")).collect(),
                1000.0 / 24.0,
                false,
            ),
        );
        animation_registry.add_anim(
            "player_turn".to_string(),
            SpriteAnimation::new(
                (1..=3).map(|i| format!("PlayerBlue_Frame_{i:02}")).collect(),
                1000.0 / 12.0,
                false,
            ),
        );

        let spritesheet_image = Image::from_path(ctx, "/spaceship_sprites.png")?;

        Ok(Self {
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
            tick_count: 0,
            status: None,
            animation_registry,
            animation_system: SpriteAnimationSystem::new(),
            spritesheet_data,
            spritesheet_instances: InstanceArray::new(ctx, spritesheet_image),
        })
    }

    // -- Input ----------------------------------------------------------------

    fn handle_keys(&mut self, ctx: &mut Context) {
        // Snapshot keys so we can mutate self freely
        let keys: Vec<KeyCode> = self.keys.iter().copied().collect();
        for key in keys {
            match key {
                KeyCode::Up    => self.player.velocity += Vec2::new(0.0, -1.0),
                KeyCode::Down  => self.player.velocity += Vec2::new(0.0,  1.0),
                KeyCode::Left  => self.player.velocity += Vec2::new(-1.0, 0.0),
                KeyCode::Right => self.player.velocity += Vec2::new( 1.0, 0.0),
                KeyCode::LShift => {
                    if self.player.alive && self.player.bullet_spacing == 0 {
                        self.player.cycle_weapons();
                        self.player.bullet_spacing += 50;
                    }
                }
                KeyCode::Space => {
                    if self.player.alive && self.player.bullet_spacing == 0 && self.no_attack_timer == 0 {
                        self.bullets.extend(self.player.shoot());
                        self.player.bullet_spacing = self.player.weapon().fire_rate();
                    }
                }
                KeyCode::Escape => ctx.request_quit(),
                _ => {}
            }
        }
    }

    // -- Physics & Collisions -------------------------------------------------

    fn handle_bullets(&mut self) {
        let no_attack = self.no_attack_timer > 0;

        // Player bullets vs enemies
        for bullet in &mut self.bullets {
            bullet.physics();
            if no_attack { continue; }
            for enemy in &mut self.enemies {
                if bullet.collides_with(enemy) {
                    bullet.alive = false;
                    enemy.health -= bullet.damage;
                    enemy.flash_frames = 5;
                }
            }
        }
        self.bullets.retain(|b| b.alive && !b.is_off_screen());

        // Enemy bullets vs player
        let player_vulnerable = self.player.is_vulnerable() && !no_attack;
        for bullet in &mut self.enemy_bullets {
            bullet.physics();
            if player_vulnerable && bullet.collides_with(&self.player) {
                bullet.alive = false;
                self.player.take_damage(bullet.damage);
            }
        }
        self.enemy_bullets.retain(|b| b.alive && !b.is_off_screen());
    }

    fn handle_enemies(&mut self) {
        let num_enemies = self.enemies.len();
        let no_attack = self.no_attack_timer > 0;

        for enemy in &mut self.enemies {
            enemy.physics();

            // Enemy shooting
            if self.player.alive && !no_attack {
                let scaled_chance = ENEMY_SHOOT_CHANCE * num_enemies * num_enemies / self.stage;
                if self.rng.gen_range(0..scaled_chance) == 0 {
                    let direction = self.player.position - enemy.position;
                    let dist = direction.length();
                    let accuracy = 1.0 / dist / num_enemies as f32;
                    let normal_sample: f32 = self.rng.sample(StandardNormal);
                    let noise = self.player.size / 2.0 * normal_sample * (1.0 - accuracy) * Vec2::ONE;
                    let velocity = (direction + noise).normalize() * 3.0;
                    self.enemy_bullets.push(Bullet::new(
                        enemy, velocity, Some(Vec2::new(0.0, 20.0)), 10.0, BulletType::Proton,
                    ));
                }
            }

            // Contact damage
            if !no_attack && self.player.is_vulnerable() && enemy.collides_with(&self.player) {
                self.player.take_damage(PLAYER_CONTACT_DAMAGE);
            }

            // Enemy death
            if enemy.health <= 0.0 {
                enemy.alive = false;
                self.player.experience +=
                    PLAYER_EXP_PER_KILL * 0.7_f32.powf(self.player.weapon().level() as f32);
            }
        }

        // Spawn explosions for dead enemies
        let dead_enemies: Vec<_> = self.enemies.iter()
            .filter(|e| !e.alive)
            .map(|e| e.position)
            .collect();
        for pos in dead_enemies {
            self.explosions.push(Explosion::new(
                pos, 64.0, &mut self.animation_system, &self.animation_registry,
            ));
        }

        self.enemies.retain(|e| e.alive);
    }

    fn handle_background(&mut self) {
        if self.stars.len() < NUM_STARS && self.rng.gen_range(0.0..1.0_f32) < 0.3 {
            let x = self.rng.gen_range(0.0..DISPLAY_WIDTH);
            let normal_sample: f32 = self.rng.sample(StandardNormal);
            let brightness: f32 = self.rng.gen_range(0.0..1.0);
            let size = 1.0 + 0.5 * normal_sample.abs();
            let speed = 0.3 + 0.3 * brightness + 0.3 * normal_sample.abs();
            self.stars.push(Star::new(
                Vec2::new(x, 0.0),
                Vec2::new(0.0, speed),
                size,
                brightness,
            ));
        }

        for star in &mut self.stars {
            if star.position.y > DISPLAY_HEIGHT {
                star.position.y = 0.0;
            }
            star.physics();
        }
    }

    // -- Wave management ------------------------------------------------------

    fn spawn_wave(&mut self) {
        self.stage += 1;
        self.no_attack_timer = WAVE_GRACE_PERIOD;

        if self.player.health < PLAYER_MIN_HEALTH_RESTORE {
            self.player.health = PLAYER_MIN_HEALTH_RESTORE;
        }

        for x in 0..ENEMIES_PER_ROW {
            for y in 0..ENEMY_ROWS {
                self.enemies.push(Enemy::new(
                    Vec2::new(80.0 + x as f32 * 110.0, 50.0 + y as f32 * 100.0),
                ));
            }
        }
    }

    fn handle_player_death(&mut self) {
        self.status = Some("game over");
        self.player.alive = false;
        self.player.health = 0.0;
        self.explosions.push(Explosion::new(
            self.player.position, 64.0,
            &mut self.animation_system, &self.animation_registry,
        ));
    }

    // -- Drawing helpers ------------------------------------------------------

    fn draw_hud_bar(
        canvas: &mut Canvas,
        ctx: &Context,
        position: Vec2,
        fill_fraction: f32,
        bar_color: Color,
        label: &str,
    ) -> GameResult {
        let bar_width = 3.0 * DISPLAY_WIDTH / 12.0;
        let bar_height = DISPLAY_HEIGHT / 24.0;

        let outline_rect = Rect::new(position.x, position.y, bar_width, bar_height);
        let filled_rect = Rect::new(position.x, position.y, fill_fraction * bar_width, bar_height);

        let outline_color = Color::new(bar_color.r, bar_color.g, bar_color.b, 0.8);
        let fill_color = Color::new(bar_color.r, bar_color.g, bar_color.b, 0.3);

        canvas.draw(
            &Mesh::new_rectangle(ctx, DrawMode::stroke(2.0), outline_rect, outline_color)?,
            DrawParam::default(),
        );
        canvas.draw(
            &Mesh::new_rectangle(ctx, DrawMode::fill(), filled_rect, fill_color)?,
            DrawParam::default(),
        );

        let text = Text::new(TextFragment::new(label));
        canvas.draw(&text, DrawParam::default().dest(position + Vec2::new(10.0, 10.0)));

        Ok(())
    }

}

fn queue_sprite(
    instances: &mut InstanceArray,
    anim_system: &SpriteAnimationSystem,
    sheet_data: &SpriteSheetData,
    obj: &dyn SpriteObject,
    base_params: DrawParam,
    dest: Vec2,
    rotation: Option<f32>,
    color: Option<Color>,
) {
    if let Some(src) = obj.get_fractional_frame(anim_system, sheet_data) {
        let mut params = base_params.src(src).dest(dest);
        if let Some(r) = rotation {
            params = params.rotation(r);
        }
        if let Some(c) = color {
            params = params.color(c);
        }
        instances.push(params);
    }
}

// =============================================================================
// EventHandler
// =============================================================================

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.tick_count += 1;

        if SHOW_FRAMERATE && self.tick_count % 60 == 0 {
            println!(
                "FPS: {:.0}, #Bullets: {}",
                ctx.time.fps(),
                self.bullets.len() + self.enemy_bullets.len()
            );
        }

        self.no_attack_timer = self.no_attack_timer.saturating_sub(1);

        self.handle_keys(ctx);
        self.handle_bullets();
        self.handle_enemies();
        self.handle_background();

        // Expire finished explosions
        for exp in &mut self.explosions {
            if exp.poll_animation_finished(&self.animation_system) {
                exp.finished = true;
            }
        }
        self.explosions.retain(|exp| !exp.finished);

        // Advance animations
        self.animation_system.time_tick(ctx.time.delta().as_millis() as f32);

        // Player physics & leveling
        self.player.physics();
        if self.player.experience >= EXP_TO_LEVEL {
            self.player.experience = 0.0;
            self.player.weapon_mut().level_up();
        }

        // Wave progression
        if self.player.alive && self.enemies.is_empty() {
            self.spawn_wave();
        }

        // Death check
        if self.player.alive && self.player.health <= 0.0 {
            self.handle_player_death();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = Canvas::from_frame(ctx, Color::BLACK);

        // In ggez 0.9, offset affects both positioning and rotation origin.
        // Use offset(0,0) for all sprites so dest = top-left, matching ggez 0.5 SpriteBatch behavior.
        let base_params = DrawParam::default();

        // -- Background stars -------------------------------------------------
        for star in &self.stars {
            let dim = star.brightness * 0.8;
            let mesh = Mesh::new_circle(
                ctx, DrawMode::fill(), star.position, star.size, 1.0,
                Color::new(dim, dim, dim, 1.0),
            )?;
            canvas.draw(&mesh, DrawParam::default());
        }

        // -- Sprites ----------------------------------------------------------

        // Player bullets
        for bullet in &self.bullets {
            queue_sprite(
                &mut self.spritesheet_instances, &self.animation_system, &self.spritesheet_data,
                bullet, base_params, bullet.position(), Some(bullet.angle), None,
            );
        }

        // Enemy bullets
        for bullet in &self.enemy_bullets {
            queue_sprite(
                &mut self.spritesheet_instances, &self.animation_system, &self.spritesheet_data,
                bullet, base_params, bullet.position(), Some(bullet.angle), None,
            );
        }

        // Enemies
        for enemy in &self.enemies {
            let color = if enemy.flash_frames > 0 {
                Some(Color::new(3.0, 0.8, 0.8, 1.0))
            } else {
                None
            };
            queue_sprite(
                &mut self.spritesheet_instances, &self.animation_system, &self.spritesheet_data,
                enemy, base_params, enemy.position(), None, color,
            );
        }

        // Explosions
        for exp in &self.explosions {
            queue_sprite(
                &mut self.spritesheet_instances, &self.animation_system, &self.spritesheet_data,
                exp, base_params, exp.position(), None, None,
            );
        }

        // Player
        if self.player.alive {
            let flash_period = PLAYER_INVINCIBILITY_FRAMES as usize / 10;
            let flashing = (self.player.invincibility_frames > 0 || self.no_attack_timer > 0)
                && self.tick_count / flash_period % 2 == 0;
            let color = if flashing {
                Some(Color::new(1.0, 1.0, 1.0, 0.1))
            } else {
                None
            };
            let player_pos = self.player.position();
            let player_frame = self.player.get_fractional_frame(&self.animation_system, &self.spritesheet_data);
            if let Some(src) = player_frame {
                let mut params = base_params.src(src).dest(player_pos);
                if let Some(c) = color {
                    params = params.color(c);
                }
                self.spritesheet_instances.push(params);
            }
        }

        canvas.draw(&self.spritesheet_instances, DrawParam::default());
        self.spritesheet_instances.clear();

        // -- Debug hitboxes ---------------------------------------------------
        if SHOW_HITBOXES {
            let hitbox_color = Color::from(HITBOX_COLOR);
            let mut draw_hitboxes = |tree: &crate::hitbox::HitboxTree| -> GameResult {
                for hb in tree.bfs_iter() {
                    let rect = Rect::new(hb.point.x, hb.point.y, hb.size.x, hb.size.y);
                    let mesh = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, hitbox_color)?;
                    canvas.draw(&mesh, DrawParam::default());
                }
                Ok(())
            };
            for b in &self.bullets { draw_hitboxes(&b.hitbox_tree)?; }
            for b in &self.enemy_bullets { draw_hitboxes(&b.hitbox_tree)?; }
            for e in &self.enemies { draw_hitboxes(&e.hitbox_tree)?; }
            if self.player.alive { draw_hitboxes(&self.player.hitbox_tree)?; }
        }

        // -- HUD --------------------------------------------------------------
        let hud_y = 23.0 * DISPLAY_HEIGHT / 24.0 - 10.0;
        let health_pos = Vec2::new(10.0, hud_y);
        let exp_pos = Vec2::new(9.0 * DISPLAY_WIDTH / 12.0 - 10.0, hud_y);

        Self::draw_hud_bar(
            &mut canvas, ctx, health_pos,
            self.player.health / PLAYER_MAX_HEALTH,
            Color::RED,
            &format!("health: {}/{}", self.player.health, PLAYER_MAX_HEALTH),
        )?;
        Self::draw_hud_bar(
            &mut canvas, ctx, exp_pos,
            self.player.experience / EXP_TO_LEVEL,
            Color::GREEN,
            &self.player.weapon().info(),
        )?;

        // Stage counter / wave timer
        let stage_label = if self.no_attack_timer > 0 {
            format!("timer: {}", self.no_attack_timer)
        } else {
            format!("stage: {}", self.stage)
        };
        canvas.draw(
            &Text::new(TextFragment::new(stage_label)),
            DrawParam::default().dest(Vec2::new(20.0, 10.0)),
        );

        // Game-over text
        if let Some(status) = self.status {
            canvas.draw(
                &Text::new(TextFragment::new(status)),
                DrawParam::default().dest(Vec2::new(DISPLAY_WIDTH / 2.0 - 50.0, DISPLAY_HEIGHT / 2.0)),
            );
        }

        canvas.finish(ctx)?;
        Ok(())
    }

    fn key_up_event(&mut self, _ctx: &mut Context, input: KeyInput) -> Result<(), GameError> {
        if let Some(keycode) = input.keycode {
            self.keys.remove(&keycode);
        }
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeat: bool) -> Result<(), GameError> {
        if let Some(keycode) = input.keycode {
            self.keys.insert(keycode);
        }
        Ok(())
    }
}
