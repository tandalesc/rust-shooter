use nalgebra as na;
use trees::tr;

use crate::state::{DISPLAY_RESOLUTION, INTERNAL_RESOLUTION, FRICTION};
use crate::hitbox::{Hitbox, HitboxTree};
use crate::weapon::{Weapon, MachineGun, WideGun};
use crate::spritesheet::{SpriteAnimationSystem, SpriteAnimationRegistry, SpriteObject};

pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;

//helper trait so we can check for collisions, positions, and sizes for any of our objects
pub trait GameObject {
    fn get_position(&self) -> Point2;
    fn get_size(&self) -> Vector2;
    //default implementation is no hitbox
    fn get_hitbox_tree(&self) -> Option<&HitboxTree> {
        None
    }
    //not all game objects have weapons
    fn get_weapon_mut(&mut self) -> Option<&mut Weapon> {
        None
    }
    fn get_weapon(&self) -> Option<&Weapon> {
        None
    }
    //default implementation -- get hitbox trees (if they exist) and do standard collision check
    fn collides_with(&self, other: &dyn GameObject) -> bool {
        if let (Some(sht), Some(oht)) = (self.get_hitbox_tree(), other.get_hitbox_tree()) {
            sht.collides_with(&oht)
        } else {
            false
        }
    }
    fn is_off_screen(&self) -> bool {
        let pos = self.get_position();
        let size = self.get_size();
        pos.x<(-size.x) || pos.x>INTERNAL_RESOLUTION.0 ||
        pos.y<(-size.y) || pos.y>INTERNAL_RESOLUTION.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub health: f32,
    pub experience: f32,
    pub invincibility_frames: u32,
    pub bullet_spacing: u32,
    weapons: Vec<Weapon>,
    pub current_weapon_idx: usize,
    pub hitbox_tree: HitboxTree,
    pub alive: bool
}
impl Player {
    pub fn new() -> Player {
        let pos = Point2::new(50.0, 400.0);
        let size = 65.0;
        Player {
            position: pos,
            velocity: Vector2::new(0.0, 0.0),
            size: size,
            health: 100.0,
            experience: 0.0,
            bullet_spacing: 0,
            invincibility_frames: 0,
            weapons: vec![
                Weapon::WideGun(WideGun::new()),
                Weapon::MachineGun(MachineGun::new())
            ],
            current_weapon_idx: 0,
            alive: true,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new_square(pos, size)) //root
                    //wings
                    /( tr(Hitbox::new(pos+Vector2::new(size/12., 7.*size/12.), Vector2::new(5.*size/6., size/4.))) )
                    //body
                    /( tr(Hitbox::new(pos+Vector2::new(size/3., size/6.), Vector2::new(size/3., 2.*size/3.))) )
            )
        }
    }
    pub fn cycle_weapons(&mut self) {
        if self.current_weapon_idx+1 < self.weapons.len() {
            self.current_weapon_idx += 1;
        } else {
            self.current_weapon_idx = 0;
        }
    }
    pub fn shoot(&self) -> Vec<Bullet> {
        self.get_weapon().unwrap().fire(&self)
    }
    pub fn physics(&mut self) {
        //rudimentary physics
        let mut new_pos = self.position + self.velocity;
        self.velocity *= 1.0-FRICTION; //take away the frictional component

        //clamp position to screen
        new_pos.x = new_pos.x.max(0.0).min(INTERNAL_RESOLUTION.0-self.size);
        new_pos.y = new_pos.y.max(0.0).min(INTERNAL_RESOLUTION.1-self.size);
        let delta = new_pos - self.position;

        //update positions of sprite and hitbox
        self.position = new_pos;
        self.hitbox_tree.move_delta(delta);

        //weapon cooldown
        if self.bullet_spacing > 0 {
            self.bullet_spacing -= 1;
        }
        //invincibility frames
        if self.invincibility_frames > 0 {
            self.invincibility_frames -= 1;
        }
    }
}
impl GameObject for Player {
    fn get_position(&self) -> Point2 {
        self.position
    }
    fn get_size(&self) -> Vector2 {
        Vector2::new(self.size, self.size)
    }
    fn get_hitbox_tree(&self) -> Option<&HitboxTree> {
        Some(&self.hitbox_tree)
    }
    fn get_weapon_mut(&mut self) -> Option<&mut Weapon> {
        Some(&mut self.weapons[self.current_weapon_idx])
    }
    fn get_weapon(&self) -> Option<&Weapon> {
        Some(&self.weapons[self.current_weapon_idx])
    }
}
impl SpriteObject for Player {
    fn get_frame(&self, sprite_system: &SpriteAnimationSystem) -> Option<String> {
        Some("PlayerBlue_Frame_01".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub health: f32,
    pub flash_frames: i32,
    pub alive: bool,
    pub hitbox_tree: HitboxTree
}
impl Enemy {
    pub fn new(position: Point2) -> Enemy {
        let size = 60.0;
        Enemy {
            position: position,
            velocity: Vector2::new(0.0, 0.03),
            size: size,
            health: 80.0,
            flash_frames: 0,
            alive: true,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new_square(position, size)) //root
                    //wings
                    /( tr(Hitbox::new(position+Vector2::new(0., 2.*size/5.), Vector2::new(size, size/5.))) )
                    //center
                    /( tr(Hitbox::new(position+Vector2::new(3.*size/10., size/5.), Vector2::new(2.*size/5., 3.*size/5.))) )
            )
        }
    }
    pub fn physics(&mut self) {
        self.position += self.velocity;
        self.hitbox_tree.move_delta(self.velocity);
        //invincibility frames
        self.flash_frames = (self.flash_frames-1).max(0);
    }
}
impl GameObject for Enemy {
    fn get_position(&self) -> Point2 {
        self.position
    }
    fn get_size(&self) -> Vector2 {
        Vector2::new(self.size, self.size)
    }
    fn get_hitbox_tree(&self) -> Option<&HitboxTree> {
        Some(&self.hitbox_tree)
    }
}
impl SpriteObject for Enemy {
    fn get_frame(&self, sprite_system: &SpriteAnimationSystem) -> Option<String> {
        Some("Enemy01_Red_Frame_1".to_string())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BulletType {
    Minigun,
    Laser,
    Proton
}

#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Point2,
    pub velocity: Vector2,
    pub angle: f32,
    pub size: f32,
    pub damage: f32,
    pub alive: bool,
    pub hitbox_tree: HitboxTree,
    pub bullet_type: BulletType
}
impl Bullet {
    pub fn new(obj: &dyn GameObject, velocity: Vector2, offset: Option<Vector2>, damage: f32, bullet_type: BulletType) -> Bullet {
        let bullet_size = 10.0;
        let default_offset = Vector2::new((obj.get_size().x-bullet_size)/2.0, velocity.y.signum()*bullet_size);
        let final_offset = if let Some(o) = offset { default_offset + o } else { default_offset };
        let pos = obj.get_position() + final_offset;
        Bullet {
            position: pos,
            velocity: velocity,
            angle: velocity.x.atan2(-velocity.y),
            size: bullet_size,
            damage: damage,
            alive: true,
            bullet_type: bullet_type,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new(pos+Vector2::new(bullet_size/8.0, 0.), Vector2::new(3.0*bullet_size/4.0, 3.0*bullet_size/4.0)))
            )
        }
    }
    pub fn physics(&mut self) {
        self.position += self.velocity;
        self.hitbox_tree.move_delta(self.velocity);
    }
}
impl GameObject for Bullet {
    fn get_position(&self) -> Point2 {
        self.position
    }
    fn get_size(&self) -> Vector2 {
        Vector2::new(self.size, self.size)
    }
    fn get_hitbox_tree(&self) -> Option<&HitboxTree> {
        Some(&self.hitbox_tree)
    }
}
impl SpriteObject for Bullet {
    fn get_frame(&self, sprite_system: &SpriteAnimationSystem) -> Option<String> {
        match self.bullet_type {
            BulletType::Minigun => Some("Minigun_Small".to_string()),
            BulletType::Laser => Some("Laser_Small".to_string()),
            BulletType::Proton => Some("Proton_Medium".to_string())
        }
    }
}

pub struct Star {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub brightness: f32
}
impl Star {
    pub fn new(position: Point2, velocity: Vector2, size: f32, brightness: f32) -> Star {
        Star {
            position: position,
            velocity: velocity,
            size: size,
            brightness: brightness
        }
    }
    pub fn physics(&mut self) {
        self.position += self.velocity;
    }
}
impl GameObject for Star {
    fn get_position(&self) -> Point2 {
        self.position
    }
    fn get_size(&self) -> Vector2 {
        Vector2::new(self.size, self.size)
    }
    fn is_off_screen(&self) -> bool {
        let pos = self.get_position();
        let size = self.get_size();
        pos.x<(-size.x) || pos.x>DISPLAY_RESOLUTION.0 ||
        pos.y<(-size.y) || pos.y>DISPLAY_RESOLUTION.1
    }
}

pub struct Explosion {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub anim_handle: usize,
    pub finished: bool
}
impl Explosion {
    pub fn new(position: Point2, velocity: Vector2, size: f32, sprite_system: &mut SpriteAnimationSystem, animation_registry: &SpriteAnimationRegistry) -> Explosion {
        let mut exp = Explosion {
            position: position,
            velocity: velocity,
            size: size,
            anim_handle: 0,
            finished: false
        };
        exp.register_in_system(sprite_system, animation_registry);
        exp
    }
    pub fn poll_animation_finished(&self, anim_system: &SpriteAnimationSystem) -> bool {
        if let Some(anim) = anim_system.get_anim(self.anim_handle) {
            anim.finished
        } else {
            false
        }
    }
    pub fn physics(&mut self) {
        self.position += self.velocity;
    }
}
impl GameObject for Explosion {
    fn get_position(&self) -> Point2 {
        self.position
    }
    fn get_size(&self) -> Vector2 {
        Vector2::new(self.size, self.size)
    }
}
impl SpriteObject for Explosion {
    fn get_frame(&self, sprite_system: &SpriteAnimationSystem) -> Option<String> {
        if let Some(frame) = sprite_system.get_frame(self.anim_handle) {
            Some(frame.clone())
        } else {
            None
        }
    }
    fn register_in_system(&mut self, sprite_system: &mut SpriteAnimationSystem, animation_registry: &SpriteAnimationRegistry) {
        self.anim_handle = sprite_system.add_registered_anim(
            "explosion".to_string(),
            animation_registry
        ).unwrap();
    }
}
