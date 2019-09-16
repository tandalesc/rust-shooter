
use trees::{bfs, Tree, Node};
use trees::linked::fully::iter::{Iter, IterMut};
use std::collections::LinkedList;

use crate::state::{RESOLUTION};
use crate::shooter::{Point2, Vector2};

//sub-divides screen into 10x10 grid for the purposes of speeding up collision detection
const GRID_RESOLUTION: (f32, f32) = (10.0, 10.0);

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
    //move bounding box by a certain amount
    pub fn move_delta(&mut self, delta: Vector2) {
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
        //we are using LinkedList as a light-weight queue (need to add to end and pop from beginning)
        let mut self_queue: LinkedList<&Node<Hitbox>> = LinkedList::new();
        let mut other_queue: LinkedList<&Node<Hitbox>> = LinkedList::new();
        //seed with root nodes
        let (self_root, other_root) = (self.tree.root(), other.tree.root());
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
                } else if other_node.degree()>0 {
                    //if the other node has children, explore those next
                    other_node.iter().for_each(|node| other_queue.push_back(node));
                    other_queue.pop_front();
                } else if self_node.degree()>0 {
                    //otherwise, explore this node's children if it has any
                    self_node.iter().for_each(|node| self_queue.push_back(node));
                    self_queue.pop_front();
                }
            } else {
                //if self_node is a leaf node, then throw it away and explore the next node
                if self_node.is_leaf() {
                    self_queue.pop_front();
                } else {
                    //otherwise, throw away the other node and explore what's next
                    other_queue.pop_front();
                }
            }
        }
        false
    }
    //helper to move all bounding boxes in this tree by a certain amount
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

//convert a Point2 to a discrete location on a grid
//returns (f32, f32) as a convinience, but all components are floored
fn get_grid_square(p: Point2) -> (f32, f32) {
    ((p.x*GRID_RESOLUTION.0/RESOLUTION.0).floor(), (p.y*GRID_RESOLUTION.1/RESOLUTION.1).floor())
}
