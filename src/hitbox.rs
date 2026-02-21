use std::collections::LinkedList;

use ggez::glam::Vec2;
use crate::config::{WORLD_WIDTH, WORLD_HEIGHT};

//sub-divides screen into 10x10 grid for the purposes of speeding up collision detection
const GRID_RESOLUTION: (f32, f32) = (10.0, 10.0);

#[derive(Debug, Clone, PartialEq)]
pub struct Hitbox {
    pub point: Vec2,
    pub size: Vec2
}
impl Hitbox {
    pub fn new(point: Vec2, size: Vec2) -> Hitbox {
        Hitbox {
            point,
            size
        }
    }
    pub fn new_square(point: Vec2, size: f32) -> Hitbox {
        Hitbox::new(point, Vec2::new(size, size))
    }
    //move bounding box by a certain amount
    pub fn move_delta(&mut self, delta: Vec2) {
        self.point += delta;
    }
    //standard bounding box collision
    pub fn collides_with(&self, other: &Hitbox) -> bool {
        self.point.x < other.point.x+other.size.x &&
        self.point.x+self.size.x > other.point.x &&
        self.point.y < other.point.y+other.size.y &&
        self.point.y+self.size.y > other.point.y
    }
}

/// A simple tree node: data + children
#[derive(Debug, Clone, PartialEq)]
pub struct HitboxNode {
    pub data: Hitbox,
    pub children: Vec<HitboxNode>
}

impl HitboxNode {
    pub fn new(data: Hitbox) -> HitboxNode {
        HitboxNode { data, children: Vec::new() }
    }
    pub fn with_child(mut self, child: HitboxNode) -> Self {
        self.children.push(child);
        self
    }
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HitboxTree {
    root: HitboxNode
}
impl HitboxTree {
    pub fn new(root: HitboxNode) -> HitboxTree {
        HitboxTree { root }
    }
    pub fn collides_with(&self, other: &HitboxTree) -> bool {
        //we are using LinkedList as a light-weight queue (need to add to end and pop from beginning)
        let mut self_queue: LinkedList<&HitboxNode> = LinkedList::new();
        let mut other_queue: LinkedList<&HitboxNode> = LinkedList::new();
        //seed with root nodes
        let (self_root, other_root) = (&self.root, &other.root);
        self_queue.push_back(self_root);
        other_queue.push_back(other_root);
        //preliminary collision
        //get location on 10x10 grid and check to see if objects are in the same grid tile
        let self_grid = get_grid_square(self_root.data.point);
        let other_grid = get_grid_square(other_root.data.point);
        if (self_grid.0-other_grid.0).abs() > 1.0 ||
            (self_grid.1-other_grid.1).abs() > 1.0 {
            return false;
        }
        //hierarchical collision
        //start at root node, and attempt to find a collision between leaf nodes
        while let (Some(self_node), Some(other_node))=(self_queue.front(), other_queue.front()) {
            //collision found! (using bounding box algorithm on current Hitbox node)
            if self_node.data.collides_with(&other_node.data) {
                //if both are leaf nodes, then we've found the most specific collision we can
                if self_node.is_leaf() && other_node.is_leaf() {
                    return true;
                } else if !other_node.children.is_empty() {
                    //if the other node has children, explore those next
                    for child in &other_node.children {
                        other_queue.push_back(child);
                    }
                    other_queue.pop_front();
                } else if !self_node.children.is_empty() {
                    //otherwise, explore this node's children if it has any
                    for child in &self_node.children {
                        self_queue.push_back(child);
                    }
                    self_queue.pop_front();
                }
            } else {
                //preferentially explore the children of the other object
                if other_queue.len()>1 {
                    other_queue.pop_front();
                } else {
                    //if there are no more objects in self_queue after this, then the comparison failed
                    self_queue.pop_front();
                }
            }
        }
        false
    }
    //helper to move all bounding boxes in this tree by a certain amount
    pub fn move_delta(&mut self, delta: Vec2) {
        Self::move_delta_recursive(&mut self.root, delta);
    }
    fn move_delta_recursive(node: &mut HitboxNode, delta: Vec2) {
        node.data.move_delta(delta);
        for child in &mut node.children {
            Self::move_delta_recursive(child, delta);
        }
    }
    pub fn bfs_iter(&self) -> Vec<&Hitbox> {
        let mut result = Vec::new();
        let mut queue = LinkedList::new();
        queue.push_back(&self.root);
        while let Some(node) = queue.pop_front() {
            result.push(&node.data);
            for child in &node.children {
                queue.push_back(child);
            }
        }
        result
    }
}

//convert a Vec2 to a discrete location on a grid
//returns (f32, f32) as a convenience, but all components are floored
fn get_grid_square(p: Vec2) -> (f32, f32) {
    ((p.x * GRID_RESOLUTION.0 / WORLD_WIDTH).floor(), (p.y * GRID_RESOLUTION.1 / WORLD_HEIGHT).floor())
}
