use nalgebra as na;

pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;

#[derive(Debug, Clone, Copy)]
pub struct Player {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub bullet_counter: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct Enemy {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32
}

#[derive(Debug, Clone, Copy)]
pub struct Bullet {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32
}
impl Bullet {
    pub fn physics(&mut self) {
        if self.position[1] > -self.size {
            self.position += self.velocity;
        }
    }
}
