use nalgebra as na;
pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;

pub const FRICTION: f32 = 0.1;
pub const BULLET_SPEED: f32 = 7.0;
pub const BULLET_SIZE: f32 = 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub bullet_spacing: i16
}
impl Player {
    pub fn new() -> Player {
        Player {
            position: Point2::new(50.0, 400.0),
            velocity: Vector2::new(0.0, 0.0),
            size: 50.0,
            bullet_spacing: 0
        }
    }
    pub fn physics(&mut self) {
        //rudimentary physics
        self.position += self.velocity;
        self.velocity -= FRICTION*self.velocity;
        //clamp position to screen
        self.position[0] = self.position[0].max(0.0).min(640.0 - self.size);
        self.position[1] = self.position[1].max(0.0).min(480.0 - self.size);
        //weapon cooldown
        self.bullet_spacing = (self.bullet_spacing-1).max(0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub health: f32
}
impl Enemy {
    pub fn new(position: Point2) -> Enemy {
        Enemy {
            position: position,
            velocity: Vector2::new(0.0, 0.04),
            size: 45.0,
            health: 100.0
        }
    }
    pub fn physics(&mut self) {
        self.position += self.velocity;
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32
}
impl Bullet {
    pub fn new(player: &Player, velocity: Vector2, offset: Option<Vector2>) -> Bullet {
        let default_offset = Vector2::new((player.size-BULLET_SIZE)/2.0, velocity.y.signum()*BULLET_SIZE);
        let final_offset = if let Some(o) = offset { default_offset + o } else { default_offset };
        Bullet {
            position: player.position + final_offset,
            velocity: velocity,
            size: BULLET_SIZE
        }
    }
    //pub fn new_center(player: &Player, direction: f32) -> Bullet {
    //    Bullet::new(player, Vector2::new(0.0, direction*BULLET_SPEED), None)
    //}
    pub fn physics(&mut self) {
        self.position += self.velocity;
    }
}
