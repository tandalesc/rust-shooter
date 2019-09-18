
use crate::shooter::{Vector2, Bullet, Player};

#[derive(Debug, Clone, PartialEq)]
pub enum Weapon {
    MachineGun(MachineGun),
    WideGun(WideGun)
}
impl Weapon {
    pub fn fire(&self, shooter: &Player) -> Vec<Bullet> {
        match self {
            Weapon::MachineGun(mg) => { mg.fire(shooter) },
            Weapon::WideGun(wg) => { wg.fire(shooter) }
        }
    }
    pub fn get_info(&self) -> String {
        match self {
            Weapon::MachineGun(mg) => { format!("MachineGun ( level: {} )", mg.level) },
            Weapon::WideGun(wg) => { format!("WideGun ( level: {} )", wg.level) }
        }
    }
    pub fn get_fire_rate(&self) -> u32 {
        match self {
            Weapon::MachineGun(mg) => { mg.fire_rate },
            Weapon::WideGun(wg) => { wg.fire_rate }
        }
    }
    pub fn get_level(&self) -> u32 {
        match self {
            Weapon::MachineGun(mg) => { mg.level },
            Weapon::WideGun(wg) => { wg.level }
        }
    }
    pub fn level_up(&mut self) {
        match self {
            Weapon::MachineGun(mg) => { mg.level_up(); },
            Weapon::WideGun(wg) => { wg.level_up(); }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MachineGun {
    pub level: u32,
    pub fire_rate: u32,
    pub fire_offset: f32,
    pub bullet_speed: f32,
    pub bullet_damage: f32
}
impl MachineGun {
    pub fn new() -> MachineGun {
        MachineGun {
            level: 0,
            fire_rate: 9,
            fire_offset: 3.0,
            bullet_speed: 5.5,
            bullet_damage: 5.0
        }
    }
    pub fn level_up(&mut self) {
        self.level += 1;
        self.fire_rate = (9-self.level/2).max(3);
        self.bullet_speed = 5.0+(self.level as f32)*1.5;
    }
    pub fn fire(&self, shooter: &Player) -> Vec<Bullet> {
        let velocity = Vector2::new(0.0, -self.bullet_speed);
        let offset = Vector2::new(0.0, self.fire_offset);
        vec![Bullet::new(shooter, velocity, Some(offset), self.bullet_damage)]
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WideGun {
    pub level: u32,
    pub fire_rate: u32,
    pub num_bullets: u32,
    pub fire_offset: f32,
    pub bullet_speed: f32,
    pub bullet_damage: f32
}
impl WideGun {
    pub fn new() -> WideGun {
        WideGun {
            level: 0,
            fire_rate: 12,
            num_bullets: 3,
            fire_offset: 3.0,
            bullet_speed: 5.0,
            bullet_damage: 2.0
        }
    }
    pub fn level_up(&mut self) {
        self.level += 1;
        self.num_bullets = 3+(self.level+2)/2;
        self.fire_rate = (12-self.level/2).max(6);
    }
    pub fn fire(&self, shooter: &Player) -> Vec<Bullet> {
        let mut bullets: Vec<Bullet> = vec![];
        let num_bullets = self.num_bullets as i32;
        let (lower_bound, upper_bound) = (-num_bullets/2, num_bullets/2+1);
        for bullet_num in lower_bound..upper_bound {
            let velocity_x = (bullet_num as f32)*0.45;
            let offset_x = (bullet_num as f32)*12.0;
            let offset_y = (bullet_num as f32).powf(2.0)*0.6-1.0;
            let velocity = Vector2::new(velocity_x, -self.bullet_speed);
            let offset = Vector2::new(offset_x, offset_y+self.fire_offset);
            bullets.push(Bullet::new(shooter, velocity, Some(offset), self.bullet_damage));
        }
        bullets
    }
}
