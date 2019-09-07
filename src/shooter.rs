use nalgebra as na;
pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;

const FRICTION: f32 = 0.08;
const BULLET_SPEED: f32 = 6.0;
const BULLET_SIZE: f32 = 8.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub bullet_counter: i16,
}
impl Player {
    pub fn new() -> Player {
        Player {
            position: Point2::new(50.0, 400.0),
            velocity: Vector2::new(0.0, 0.0),
            size: 30.0,
            bullet_counter: 0
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
        self.bullet_counter = (self.bullet_counter-1).max(0);
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
            velocity: Vector2::new(0.0, 0.0),
            size: 30.0,
            health: 100.0
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32
}
impl Bullet {
    pub fn new(player: &Player, direction: f32) -> Bullet {
        Bullet {
            position: player.position + Vector2::new(player.size/2.0 - BULLET_SIZE/2.0, direction*BULLET_SIZE),
            velocity: Vector2::new(0.0, direction*BULLET_SPEED),
            size: BULLET_SIZE
        }
    }
    pub fn physics(&mut self) {
        if self.position[1] > -self.size {
            self.position += self.velocity;
        }
    }
}
