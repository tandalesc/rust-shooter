use nalgebra as na;
use trees::tr;

use crate::state::{RESOLUTION, FRICTION};
use crate::hitbox::{Hitbox, HitboxTree};
use crate::weapon::{Weapon, MachineGun, WideGun};

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
        pos.x<(-size.x) || pos.x>RESOLUTION.0 ||
        pos.y<(-size.y) || pos.y>RESOLUTION.1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub health: f32,
    pub experience: f32,
    pub weapon_level: u32,
    pub invincibility_frames: u32,
    pub bullet_spacing: u32,
    pub weapon: Weapon,
    pub hitbox_tree: HitboxTree,
}
impl Player {
    pub fn new() -> Player {
        let pos = Point2::new(50.0, 400.0);
        let size = 50.0;
        Player {
            position: pos,
            velocity: Vector2::new(0.0, 0.0),
            size: size,
            health: 100.0,
            experience: 0.0,
            weapon_level: 0,
            bullet_spacing: 0,
            invincibility_frames: 0,
            weapon: Weapon::WideGun(WideGun::new()),//Weapon::MachineGun(MachineGun::new()),
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new_square(pos, size)) //root
                    /( tr(Hitbox::new(pos+Vector2::new(size/7.0, size/3.0), Vector2::new(5.0*size/7.0, size/3.0))) ) //fuselage
                    /( tr(Hitbox::new(pos+Vector2::new(size/20.0, 2.0*size/5.0), Vector2::new(18.0*size/20.0, size/5.0))) ) //wings
                    /( tr(Hitbox::new(pos+Vector2::new(size/3.0, size/7.0), Vector2::new(size/3.0, 5.0*size/7.0))) ) //engines
            )
        }
    }
    pub fn shoot(&self) -> Vec<Bullet> {
        self.weapon.fire(&self)
    }
    pub fn physics(&mut self) {
        //rudimentary physics
        let new_pos = self.position + self.velocity;
        self.velocity -= FRICTION*self.velocity;

        //clamp position to screen
        let delta = Vector2::new(
            new_pos.x.max(0.0).min(RESOLUTION.0-self.size) - self.position.x,
            new_pos.y.max(0.0).min(RESOLUTION.1-self.size) - self.position.y
        );

        //update positions of sprite and hitbox
        self.position += delta;
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub health: f32,
    pub flash_frames: i32,
    pub hitbox_tree: HitboxTree
}
impl Enemy {
    pub fn new(position: Point2) -> Enemy {
        let size = 45.0;
        Enemy {
            position: position,
            velocity: Vector2::new(0.0, 0.04),
            size: size,
            health: 80.0,
            flash_frames: 0,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new_square(position, size)) //root
                    /( tr(Hitbox::new(position+Vector2::new(0.0, 2.0*size/5.0), Vector2::new(size, size/7.0))) ) //wings
                    /( tr(Hitbox::new(position+Vector2::new(size/3.0, 0.0), Vector2::new(size/3.0, 7.0*size/10.0))) ) //fuselage
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


#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Point2,
    pub velocity: Vector2,
    pub angle: f32,
    pub size: f32,
    pub damage: f32,
    pub hitbox_tree: HitboxTree
}
impl Bullet {
    pub fn new(obj: &dyn GameObject, velocity: Vector2, offset: Option<Vector2>, damage: f32) -> Bullet {
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
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new(pos+Vector2::new(bullet_size/8.0, bullet_size/8.0), Vector2::new(3.0*bullet_size/4.0, 3.0*bullet_size/4.0)))
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
