use ggez::glam::Vec2;

use crate::config::*;
use crate::hitbox::{Hitbox, HitboxTree, HitboxNode};
use crate::weapon::{Weapon, MachineGun, WideGun};
use crate::spritesheet::{SpriteAnimationSystem, SpriteAnimationRegistry, SpriteObject};

/// Anything with a position and size that can participate in collision detection.
pub trait GameObject {
    fn position(&self) -> Vec2;
    fn size(&self) -> Vec2;

    fn hitbox_tree(&self) -> Option<&HitboxTree> { None }

    fn collides_with(&self, other: &dyn GameObject) -> bool {
        match (self.hitbox_tree(), other.hitbox_tree()) {
            (Some(a), Some(b)) => a.collides_with(b),
            _ => false,
        }
    }

    fn is_off_screen(&self) -> bool {
        let pos = self.position();
        let size = self.size();
        pos.x < -size.x || pos.x > WORLD_WIDTH
            || pos.y < -size.y || pos.y > WORLD_HEIGHT
    }
}

// =============================================================================
// Player
// =============================================================================

const PLAYER_SIZE: f32 = 65.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: f32,
    pub health: f32,
    pub experience: f32,
    pub invincibility_frames: u32,
    pub bullet_spacing: u32,
    pub hitbox_tree: HitboxTree,
    pub alive: bool,
    weapons: Vec<Weapon>,
    current_weapon_idx: usize,
}

impl Player {
    pub fn new() -> Self {
        let pos = Vec2::new(50.0, 400.0);
        let s = PLAYER_SIZE;
        Self {
            position: pos,
            velocity: Vec2::ZERO,
            size: s,
            health: PLAYER_MAX_HEALTH,
            experience: 0.0,
            bullet_spacing: 0,
            invincibility_frames: 0,
            weapons: vec![
                Weapon::WideGun(WideGun::new()),
                Weapon::MachineGun(MachineGun::new()),
            ],
            current_weapon_idx: 0,
            alive: true,
            hitbox_tree: HitboxTree::new(
                HitboxNode::new(Hitbox::new_square(pos, s))
                    .with_child(HitboxNode::new(Hitbox::new(
                        pos + Vec2::new(s / 12.0, 7.0 * s / 12.0),
                        Vec2::new(5.0 * s / 6.0, s / 4.0),
                    )))
                    .with_child(HitboxNode::new(Hitbox::new(
                        pos + Vec2::new(s / 3.0, s / 6.0),
                        Vec2::new(s / 3.0, 2.0 * s / 3.0),
                    ))),
            ),
        }
    }

    pub fn cycle_weapons(&mut self) {
        self.current_weapon_idx = (self.current_weapon_idx + 1) % self.weapons.len();
    }

    pub fn weapon(&self) -> &Weapon {
        &self.weapons[self.current_weapon_idx]
    }

    pub fn weapon_mut(&mut self) -> &mut Weapon {
        &mut self.weapons[self.current_weapon_idx]
    }

    pub fn shoot(&self) -> Vec<Bullet> {
        self.weapon().fire(self)
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.health -= amount;
        self.invincibility_frames = PLAYER_INVINCIBILITY_FRAMES;
    }

    pub fn is_vulnerable(&self) -> bool {
        self.alive && self.invincibility_frames == 0
    }

    pub fn physics(&mut self) {
        let mut new_pos = self.position + self.velocity;
        self.velocity *= 1.0 - FRICTION;

        new_pos.x = new_pos.x.clamp(0.0, WORLD_WIDTH - self.size);
        new_pos.y = new_pos.y.clamp(0.0, WORLD_HEIGHT - self.size);
        let delta = new_pos - self.position;

        self.position = new_pos;
        self.hitbox_tree.move_delta(delta);

        self.bullet_spacing = self.bullet_spacing.saturating_sub(1);
        self.invincibility_frames = self.invincibility_frames.saturating_sub(1);
    }
}

impl GameObject for Player {
    fn position(&self) -> Vec2 { self.position }
    fn size(&self) -> Vec2 { Vec2::splat(self.size) }
    fn hitbox_tree(&self) -> Option<&HitboxTree> { Some(&self.hitbox_tree) }
}

impl SpriteObject for Player {
    fn get_frame(&self, _: &SpriteAnimationSystem) -> Option<String> {
        Some("PlayerBlue_Frame_01".to_string())
    }
}

// =============================================================================
// Enemy
// =============================================================================

const ENEMY_SIZE: f32 = 60.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: f32,
    pub health: f32,
    pub flash_frames: i32,
    pub alive: bool,
    pub hitbox_tree: HitboxTree,
}

impl Enemy {
    pub fn new(position: Vec2) -> Self {
        let s = ENEMY_SIZE;
        Self {
            position,
            velocity: Vec2::new(0.0, 0.03),
            size: s,
            health: 80.0,
            flash_frames: 0,
            alive: true,
            hitbox_tree: HitboxTree::new(
                HitboxNode::new(Hitbox::new_square(position, s))
                    .with_child(HitboxNode::new(Hitbox::new(
                        position + Vec2::new(0.0, 2.0 * s / 5.0),
                        Vec2::new(s, s / 5.0),
                    )))
                    .with_child(HitboxNode::new(Hitbox::new(
                        position + Vec2::new(3.0 * s / 10.0, s / 5.0),
                        Vec2::new(2.0 * s / 5.0, 3.0 * s / 5.0),
                    ))),
            ),
        }
    }

    pub fn physics(&mut self) {
        self.position += self.velocity;
        self.hitbox_tree.move_delta(self.velocity);
        self.flash_frames = (self.flash_frames - 1).max(0);
    }
}

impl GameObject for Enemy {
    fn position(&self) -> Vec2 { self.position }
    fn size(&self) -> Vec2 { Vec2::splat(self.size) }
    fn hitbox_tree(&self) -> Option<&HitboxTree> { Some(&self.hitbox_tree) }
}

impl SpriteObject for Enemy {
    fn get_frame(&self, _: &SpriteAnimationSystem) -> Option<String> {
        Some("Enemy01_Red_Frame_1".to_string())
    }
}

// =============================================================================
// Bullet
// =============================================================================

#[derive(Debug, Clone, PartialEq)]
pub enum BulletType {
    Minigun,
    Laser,
    Proton,
}

const BULLET_SIZE: f32 = 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Vec2,
    pub velocity: Vec2,
    pub angle: f32,
    pub size: f32,
    pub damage: f32,
    pub alive: bool,
    pub hitbox_tree: HitboxTree,
    pub bullet_type: BulletType,
}

impl Bullet {
    pub fn new(
        origin: &dyn GameObject,
        velocity: Vec2,
        offset: Option<Vec2>,
        damage: f32,
        bullet_type: BulletType,
    ) -> Self {
        let default_offset = Vec2::new(
            (origin.size().x - BULLET_SIZE) / 2.0,
            velocity.y.signum() * BULLET_SIZE,
        );
        let final_offset = default_offset + offset.unwrap_or(Vec2::ZERO);
        let pos = origin.position() + final_offset;

        Self {
            position: pos,
            velocity,
            angle: velocity.x.atan2(-velocity.y),
            size: BULLET_SIZE,
            damage,
            alive: true,
            bullet_type,
            hitbox_tree: HitboxTree::new(HitboxNode::new(Hitbox::new(
                pos + Vec2::new(BULLET_SIZE / 8.0, 0.0),
                Vec2::splat(3.0 * BULLET_SIZE / 4.0),
            ))),
        }
    }

    pub fn physics(&mut self) {
        self.position += self.velocity;
        self.hitbox_tree.move_delta(self.velocity);
    }
}

impl GameObject for Bullet {
    fn position(&self) -> Vec2 { self.position }
    fn size(&self) -> Vec2 { Vec2::splat(self.size) }
    fn hitbox_tree(&self) -> Option<&HitboxTree> { Some(&self.hitbox_tree) }
}

impl SpriteObject for Bullet {
    fn get_frame(&self, _: &SpriteAnimationSystem) -> Option<String> {
        Some(match self.bullet_type {
            BulletType::Minigun => "Minigun_Small",
            BulletType::Laser => "Laser_Small",
            BulletType::Proton => "Proton_Medium",
        }.to_string())
    }
}

// =============================================================================
// Star (background decoration)
// =============================================================================

pub struct Star {
    pub position: Vec2,
    pub velocity: Vec2,
    pub size: f32,
    pub brightness: f32,
}

impl Star {
    pub fn new(position: Vec2, velocity: Vec2, size: f32, brightness: f32) -> Self {
        Self { position, velocity, size, brightness }
    }

    pub fn physics(&mut self) {
        self.position += self.velocity;
    }
}

impl GameObject for Star {
    fn position(&self) -> Vec2 { self.position }
    fn size(&self) -> Vec2 { Vec2::splat(self.size) }

    fn is_off_screen(&self) -> bool {
        let pos = self.position();
        let size = self.size();
        pos.x < -size.x || pos.x > DISPLAY_WIDTH
            || pos.y < -size.y || pos.y > DISPLAY_HEIGHT
    }
}

// =============================================================================
// Explosion
// =============================================================================

pub struct Explosion {
    pub position: Vec2,
    pub size: f32,
    pub anim_handle: usize,
    pub finished: bool,
}

impl Explosion {
    pub fn new(
        position: Vec2,
        size: f32,
        sprite_system: &mut SpriteAnimationSystem,
        animation_registry: &SpriteAnimationRegistry,
    ) -> Self {
        let mut exp = Self {
            position,
            size,
            anim_handle: 0,
            finished: false,
        };
        exp.register_in_system(sprite_system, animation_registry);
        exp
    }

    pub fn poll_animation_finished(&self, anim_system: &SpriteAnimationSystem) -> bool {
        anim_system.get_anim(self.anim_handle)
            .map_or(false, |anim| anim.finished)
    }
}

impl GameObject for Explosion {
    fn position(&self) -> Vec2 { self.position }
    fn size(&self) -> Vec2 { Vec2::splat(self.size) }
}

impl SpriteObject for Explosion {
    fn get_frame(&self, sprite_system: &SpriteAnimationSystem) -> Option<String> {
        sprite_system.get_frame(self.anim_handle).cloned()
    }

    fn register_in_system(
        &mut self,
        sprite_system: &mut SpriteAnimationSystem,
        animation_registry: &SpriteAnimationRegistry,
    ) {
        self.anim_handle = sprite_system
            .add_registered_anim("explosion".to_string(), animation_registry)
            .unwrap();
    }
}
