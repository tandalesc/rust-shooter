use ggez::glam::Vec2;

use crate::shooter::{Bullet, BulletType, GameObject};

pub trait WeaponBehavior {
    fn fire(&self, shooter: &dyn GameObject) -> Vec<Bullet>;
    fn fire_rate(&self) -> u32;
    fn level(&self) -> u32;
    fn name(&self) -> &'static str;
    fn level_up(&mut self);

    fn info(&self) -> String {
        format!("{} ( level: {} )", self.name(), self.level())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Weapon {
    MachineGun(MachineGun),
    WideGun(WideGun),
}

impl Weapon {
    fn inner(&self) -> &dyn WeaponBehavior {
        match self {
            Weapon::MachineGun(w) => w,
            Weapon::WideGun(w) => w,
        }
    }
    fn inner_mut(&mut self) -> &mut dyn WeaponBehavior {
        match self {
            Weapon::MachineGun(w) => w,
            Weapon::WideGun(w) => w,
        }
    }
    pub fn fire(&self, shooter: &dyn GameObject) -> Vec<Bullet> { self.inner().fire(shooter) }
    pub fn fire_rate(&self) -> u32 { self.inner().fire_rate() }
    pub fn level(&self) -> u32 { self.inner().level() }
    pub fn info(&self) -> String { self.inner().info() }
    pub fn level_up(&mut self) { self.inner_mut().level_up(); }
}

// -- MachineGun ---------------------------------------------------------------

const MACHINEGUN_DEFAULT_FIRERATE: u32 = 11;
const MACHINEGUN_MAX_FIRERATE: u32 = 6;
const MACHINEGUN_MAX_WIDTH: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct MachineGun {
    level: u32,
    fire_rate: u32,
    fire_offset: f32,
    bullet_speed: f32,
    bullet_damage: f32,
    pattern_width: u32,
}

impl MachineGun {
    pub fn new() -> Self {
        Self {
            level: 0,
            fire_rate: MACHINEGUN_DEFAULT_FIRERATE,
            fire_offset: 3.0,
            bullet_speed: 5.0,
            bullet_damage: 4.0,
            pattern_width: 0,
        }
    }
}

impl WeaponBehavior for MachineGun {
    fn fire(&self, shooter: &dyn GameObject) -> Vec<Bullet> {
        let velocity = Vec2::new(0.0, -self.bullet_speed);
        let w = self.pattern_width as i32;
        (-w..=w)
            .map(|i| {
                let n = i as f32;
                let offset = Vec2::new(n * 10.0, n.abs() * 2.5 - 1.0 + self.fire_offset);
                Bullet::new(shooter, velocity, Some(offset), self.bullet_damage, BulletType::Minigun)
            })
            .collect()
    }

    fn fire_rate(&self) -> u32 { self.fire_rate }
    fn level(&self) -> u32 { self.level }
    fn name(&self) -> &'static str { "MachineGun" }

    fn level_up(&mut self) {
        self.level += 1;
        if self.level > 1 && (self.level - 2) % 4 == 0 {
            self.pattern_width = (self.pattern_width + 1).min(MACHINEGUN_MAX_WIDTH);
        }
        if self.level % 2 == 0 {
            self.fire_rate = (self.fire_rate - 1).max(MACHINEGUN_MAX_FIRERATE);
        }
    }
}

// -- WideGun ------------------------------------------------------------------

const WIDEGUN_DEFAULT_FIRERATE: u32 = 14;
const WIDEGUN_MAX_FIRERATE: u32 = 10;
const WIDEGUN_MAX_WIDTH: u32 = 5;

#[derive(Debug, Clone, PartialEq)]
pub struct WideGun {
    level: u32,
    fire_rate: u32,
    num_bullets: u32,
    fire_offset: f32,
    bullet_speed: f32,
    bullet_damage: f32,
}

impl WideGun {
    pub fn new() -> Self {
        Self {
            level: 0,
            fire_rate: WIDEGUN_DEFAULT_FIRERATE,
            num_bullets: 1,
            fire_offset: 3.0,
            bullet_speed: 4.5,
            bullet_damage: 3.0,
        }
    }
}

impl WeaponBehavior for WideGun {
    fn fire(&self, shooter: &dyn GameObject) -> Vec<Bullet> {
        let n = self.num_bullets as i32;
        (-n..=n)
            .map(|i| {
                let f = i as f32;
                let velocity = Vec2::new(f * 0.4, -self.bullet_speed);
                let offset = Vec2::new(f * 10.0, f.powi(2) * 1.5 - 1.0 + self.fire_offset);
                Bullet::new(shooter, velocity, Some(offset), self.bullet_damage, BulletType::Laser)
            })
            .collect()
    }

    fn fire_rate(&self) -> u32 { self.fire_rate }
    fn level(&self) -> u32 { self.level }
    fn name(&self) -> &'static str { "WideGun" }

    fn level_up(&mut self) {
        self.level += 1;
        if self.level > 1 && (self.level - 2) % 4 == 0 {
            self.num_bullets = (self.num_bullets + 1).min(WIDEGUN_MAX_WIDTH);
        }
        if self.level % 2 == 0 {
            self.fire_rate = (self.fire_rate - 1).max(WIDEGUN_MAX_FIRERATE);
        }
    }
}
