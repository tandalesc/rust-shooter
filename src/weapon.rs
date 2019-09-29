
use crate::shooter::{Vector2, Bullet, BulletType, Player};

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

const MACHINEGUN_DEFAULT_FIRERATE: u32 = 11;
const MACHINEGUN_MAX_FIRERATE: u32 = 6;
const MACHINEGUN_DEFAULT_WIDTH: u32 = 0;
const MACHINEGUN_MAX_WIDTH: u32 = 3;
#[derive(Debug, Clone, PartialEq)]
pub struct MachineGun {
    pub level: u32,
    pub fire_rate: u32,
    pub fire_offset: f32,
    pub bullet_speed: f32,
    pub bullet_damage: f32,
    pub pattern_width: u32
}
impl MachineGun {
    pub fn new() -> MachineGun {
        MachineGun {
            level: 0,
            fire_rate: MACHINEGUN_DEFAULT_FIRERATE,
            fire_offset: 3.,
            bullet_speed: 5.,
            bullet_damage: 4.,
            pattern_width: MACHINEGUN_DEFAULT_WIDTH
        }
    }
    pub fn level_up(&mut self) {
        self.level += 1;
        //only update spread every 4 levels, starting at level 2
        if self.level>1 && (self.level-2)%4==0 {
            self.pattern_width = (self.pattern_width+1).min(MACHINEGUN_MAX_WIDTH);
        }
        if self.level%2==0 {
            self.fire_rate = (self.fire_rate-1).max(MACHINEGUN_MAX_FIRERATE);
        }
    }
    pub fn fire(&self, shooter: &Player) -> Vec<Bullet> {
        let mut bullets: Vec<Bullet> = vec![];
        let velocity = Vector2::new(0., -self.bullet_speed);
        let offset = Vector2::new(0., self.fire_offset);
        let pattern_width = self.pattern_width as i32;
        for bullet_num in -pattern_width..pattern_width+1 {
            let offset_x = (bullet_num as f32)*10.;
            let offset_y = (bullet_num as f32).abs()*2.5-1.;
            let offset = Vector2::new(offset_x, offset_y+self.fire_offset);
            bullets.push(Bullet::new(shooter, velocity, Some(offset), self.bullet_damage, BulletType::Minigun))
        }
        bullets
    }
}

const WIDEGUN_DEFAULT_FIRERATE: u32 = 14;
const WIDEGUN_MAX_FIRERATE: u32 = 10;
const WIDEGUN_DEFAULT_WIDTH: u32 = 1;
const WIDEGUN_MAX_WIDTH: u32 = 5;
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
            fire_rate: WIDEGUN_DEFAULT_FIRERATE,
            num_bullets: WIDEGUN_DEFAULT_WIDTH,
            fire_offset: 3.,
            bullet_speed: 4.5,
            bullet_damage: 3.
        }
    }
    pub fn level_up(&mut self) {
        self.level += 1;
        //only update spread every 3 levels, starting at level 2
        if self.level>1 && (self.level-2)%4==0 {
            self.num_bullets = (self.num_bullets+1).min(WIDEGUN_MAX_WIDTH);
        }
        if self.level%2==0 {
            self.fire_rate = (self.fire_rate-1).max(WIDEGUN_MAX_FIRERATE);
        }
    }
    pub fn fire(&self, shooter: &Player) -> Vec<Bullet> {
        let mut bullets: Vec<Bullet> = vec![];
        let num_bullets = self.num_bullets as i32;
        for bullet_num in -num_bullets..num_bullets+1 {
            let velocity_x = (bullet_num as f32)*0.4;
            let offset_x = (bullet_num as f32)*10.;
            let offset_y = (bullet_num as f32).powf(2.)*1.5-1.;
            let velocity = Vector2::new(velocity_x, -self.bullet_speed);
            let offset = Vector2::new(offset_x, offset_y+self.fire_offset);
            bullets.push(Bullet::new(shooter, velocity, Some(offset), self.bullet_damage, BulletType::Laser));
        }
        bullets
    }
}
