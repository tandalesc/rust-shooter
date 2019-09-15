use nalgebra as na;
use trees::{tr, bfs, Tree, Node};
use trees::linked::fully::iter::{Iter, IterMut};
use std::collections::LinkedList;

use crate::state::{RESOLUTION, GRID_RESOLUTION};

pub type Point2 = na::Point2<f32>;
pub type Vector2 = na::Vector2<f32>;

pub const FRICTION: f32 = 0.1;
pub const BULLET_SPEED: f32 = 3.0;
pub const BULLET_SIZE: f32 = 10.0;

#[derive(Debug, Clone, PartialEq)]
pub struct Hitbox {
    pub point: Point2,
    pub size: Vector2
}
impl Hitbox {
    pub fn new(point: Point2, size: Vector2) -> Hitbox {
        Hitbox {
            point: point,
            size: size
        }
    }
    pub fn new_square(point: Point2, size: f32) -> Hitbox {
        Hitbox::new(point, Vector2::new(size, size))
    }
    pub fn move_delta(&mut self, delta: Vector2) {
        self.point += delta;
    }
    pub fn collides_with(&self, other: &Hitbox) -> bool {
        self.point.x < other.point.x+other.size.x &&
        self.point.x+self.size.x > other.point.x &&
        self.point.y < other.point.y+other.size.y &&
        self.point.y+self.size.y > other.point.y
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitboxTree {
    tree: Tree<Hitbox>
}
impl HitboxTree {
    pub fn new(tree: Tree<Hitbox>) -> HitboxTree {
        HitboxTree {
            tree: tree
        }
    }
    pub fn collides_with(&self, other: &HitboxTree) -> bool {
        let mut self_queue: LinkedList<&Node<Hitbox>> = LinkedList::new();
        let mut other_queue: LinkedList<&Node<Hitbox>> = LinkedList::new();
        let (self_root, other_root) = (self.tree.root(), other.tree.root());
        self_queue.push_back(self_root);
        other_queue.push_back(other_root);
        //preliminary collision
        let self_grid = get_grid_square(self_root.data.point);
        let other_grid = get_grid_square(other_root.data.point);
        if (self_grid.0-other_grid.0).abs() > 1.0 ||
            (self_grid.1-other_grid.1).abs() > 1.0 {
            return false;
        }
        //hierarchical collision
        while let (Some(self_node), Some(other_node))=(self_queue.front(), other_queue.front()) {
            if self_node.data.collides_with(&other_node.data) {
                if self_node.is_leaf() && other_node.is_leaf() {
                    return true;
                } else if other_node.degree()>0 {
                    other_node.iter().for_each(|node| other_queue.push_back(node));
                    other_queue.pop_front();
                } else if self_node.degree()>0 {
                    self_node.iter().for_each(|node| self_queue.push_back(node));
                    self_queue.pop_front();
                }
            } else {
                if other_node.is_leaf() {
                    other_queue.pop_front();
                }
                if self_node.is_leaf() {
                    self_queue.pop_front();
                }
            }
        }
        false
    }
    pub fn move_delta(&mut self, delta: Vector2) {
        self.bfs_iter_mut().for_each(|hitbox_visit| hitbox_visit.data.move_delta(delta));
    }
    pub fn bfs_iter(&self) -> bfs::Splitted<Iter<Hitbox>> {
        self.tree.root().bfs().iter
    }
    fn bfs_iter_mut(&mut self) -> bfs::Splitted<IterMut<Hitbox>> {
        self.tree.root_mut().bfs_mut().iter
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Player {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub bullet_spacing: i16,
    pub hitbox_tree: HitboxTree
}
impl Player {
    pub fn new() -> Player {
        let pos = Point2::new(50.0, 400.0);
        let size = 50.0;
        Player {
            position: pos,
            velocity: Vector2::new(0.0, 0.0),
            size: size,
            bullet_spacing: 0,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new_square(pos, size))
                    /( tr(Hitbox::new(pos+Vector2::new(0.0, size/3.0), Vector2::new(size, size/3.0))) )
                    /( tr(Hitbox::new(pos+Vector2::new(size/4.0, size/7.0), Vector2::new(size/2.0, 3.0*size/4.0))) )
            )
        }
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
        self.bullet_spacing = (self.bullet_spacing-1).max(0);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Enemy {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub health: f32,
    pub hitbox_tree: HitboxTree
}
impl Enemy {
    pub fn new(position: Point2) -> Enemy {
        let size = 45.0;
        Enemy {
            position: position,
            velocity: Vector2::new(0.0, 0.04),
            size: size,
            health: 100.0,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new_square(position, size))
                    /( tr(Hitbox::new(position+Vector2::new(0.0, size/3.0), Vector2::new(size, size/4.0))) )
                    /( tr(Hitbox::new(position+Vector2::new(size/4.0, 0.0), Vector2::new(size/2.0, 3.0*size/4.0))) )
            )
        }
    }
    pub fn physics(&mut self) {
        self.position += self.velocity;
        self.hitbox_tree.move_delta(self.velocity);
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Bullet {
    pub position: Point2,
    pub velocity: Vector2,
    pub size: f32,
    pub hitbox_tree: HitboxTree
}
impl Bullet {
    pub fn new(player: &Player, velocity: Vector2, offset: Option<Vector2>) -> Bullet {
        let default_offset = Vector2::new((player.size-BULLET_SIZE)/2.0, velocity.y.signum()*BULLET_SIZE);
        let final_offset = if let Some(o) = offset { default_offset + o } else { default_offset };
        let pos = player.position + final_offset;
        Bullet {
            position: pos,
            velocity: velocity,
            size: BULLET_SIZE,
            hitbox_tree: HitboxTree::new(
                tr(Hitbox::new(pos+Vector2::new(BULLET_SIZE/4.0, BULLET_SIZE/6.0), Vector2::new(BULLET_SIZE/2.0, BULLET_SIZE/2.0)))
            )
        }
    }
    //pub fn new_center(player: &Player, direction: f32) -> Bullet {
    //    Bullet::new(player, Vector2::new(0.0, direction*BULLET_SPEED), None)
    //}
    pub fn physics(&mut self) {
        self.position += self.velocity;
        self.hitbox_tree.move_delta(self.velocity);
    }
}

fn get_grid_square(p: Point2) -> (f32, f32) {
    ((p.x*GRID_RESOLUTION.0/RESOLUTION.0).floor(), (p.y*GRID_RESOLUTION.1/RESOLUTION.1).floor())
}
