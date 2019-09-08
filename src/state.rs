use ggez::graphics::{Color, DrawMode, DrawParam, Image};
use ggez::event::{self, EventHandler, KeyCode, KeyMods};
use ggez::*;
use uuid::Uuid;
use std::collections::HashSet;
use std::collections::HashMap;

use crate::shooter::{Vector2, Point2, Player, Enemy, Bullet};

const BULLET_COLOR: (u8,u8,u8,u8) = (200, 200, 200, 255);
const BULLET_DAMAGE: f32 = 40.0;
const BULLET_COOLDOWN: i16 = 20;
const ENEMY_COLOR: (u8,u8,u8,u8) = (170, 50, 50, 255);
const ENEMIES: (u8,u8) = (8, 3);

pub struct Assets {
    pub image_map: HashMap<String, Image>
}
impl Assets {
    pub fn new(ctx: &mut Context) -> Assets {
        let mut assets = Assets {
            image_map: HashMap::new()
        };
        assets.image_map.insert("player".to_string(), graphics::Image::new(ctx, "/player.png").unwrap());
        assets
    }
}

pub struct State {
    player: Player,
    bullets: HashMap<uuid::Uuid, Bullet>,
    enemies: HashMap<uuid::Uuid, Enemy>,
    keys: HashSet<KeyCode>,
    assets: Assets
}

impl State {
    pub fn new(ctx: &mut Context) -> GameResult<State> {
        let mut state = State {
            player: Player::new(),
            bullets: HashMap::new(),
            enemies: HashMap::new(),
            keys: HashSet::with_capacity(6),
            assets: Assets::new(ctx)
        };
        //generate a grid of enemies
        for x in 0..ENEMIES.0 {
            for y in 0..ENEMIES.1 {
                state.enemies.insert(
                    Uuid::new_v4(),
                    Enemy::new(
                        Point2::new(80.0, 50.0) +
                        (x as f32)*Vector2::new(60.0, 0.0) +
                        (y as f32)*Vector2::new(0.0, 60.0)
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
                    if self.player.bullet_counter == 0 {
                        self.bullets.insert(Uuid::new_v4(), Bullet::new(&self.player, -1.0));
                        self.player.bullet_counter = BULLET_COOLDOWN;
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
        for (bullet_id, bullet) in &mut self.bullets {
            if (bullet.position[0]<(-bullet.size) || bullet.position[0]>(640.0)) ||
                (bullet.position[1]<(-bullet.size) || bullet.position[1]>(480.0))
            { //mark bullet for deletion
                bullet_ids.insert(*bullet_id);
            } else {
                bullet.physics();
                for (_, enemy) in &mut self.enemies {
                    let dist = bullet.position - enemy.position;
                    if dist.norm() < 100.0 { //broad-form collision
                        if bullet.position[0] < enemy.position[0]+enemy.size &&
                            bullet.position[0] + bullet.size > enemy.position[0] &&
                            bullet.position[1] < enemy.position[1]+enemy.size &&
                            bullet.position[1] + bullet.size > enemy.position[1]
                        { //narrow-form collision
                            bullet_ids.insert(*bullet_id); //mark bullet for deletion
                            enemy.health -= BULLET_DAMAGE; //do damage
                        }
                    }
                }
            }
        }
        //remove any bullets that were marked for deletion
        self.bullets.retain(|bullet_id, _| !bullet_ids.contains(bullet_id));
        //remove any enemies that died
        self.enemies.retain(|_, enemy| enemy.health > 0.0);
        Ok(())
    }
}

impl EventHandler for State {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        if self.enemies.len() == 0 {
            println!("\nyou win!\n");
            event::quit(ctx);
        }
        self.player.physics();
        for (_, enemy) in &mut self.enemies {
            enemy.physics();
        }
        self.handle_bullets(ctx)?;
        self.handle_keys(ctx)?;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::BLACK);

        //render bullets
        for (_, bullet) in &mut self.bullets {
            let rect = graphics::Rect::new(bullet.position[0], bullet.position[1], bullet.size, bullet.size);
            let r1 = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, Color::from(BULLET_COLOR))?;
            graphics::draw(ctx, &r1, DrawParam::default())?;
        }

        //render enemies
        for (_, enemy) in &mut self.enemies {
            let mut mod_color = ENEMY_COLOR.clone();
            mod_color.3 = ((mod_color.3 as f32)*(enemy.health/100.0)) as u8;
            let rect = graphics::Rect::new(enemy.position[0], enemy.position[1], enemy.size, enemy.size);
            let r1 = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, Color::from(mod_color))?;
            graphics::draw(ctx, &r1, DrawParam::default())?;
        }

        //render player
        //let rect = graphics::Rect::new(self.player.position[0], self.player.position[1], self.player.size, self.player.size);
        //let r1 = graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, graphics::WHITE)?;
        //graphics::draw(ctx, &r1, DrawParam::default())?;
        let color = Color::from((255,255,255,255));
        let sprite = self.assets.image_map.get("player").unwrap();
        graphics::draw(ctx, sprite, (self.player.position, 0.0, color))?;

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
