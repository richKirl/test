use crate::Aabb;
#[derive(Default)]
pub struct Node {
    pub bbox: Aabb,
    pub object_index: i32, // = -1;
    pub parent_index: i32, // = -1;
    pub child1: i32,       // = -1,
    pub child2: i32,       // = -1,
    pub height: i32,       // = 0;
    pub is_leaf: bool,     // = false;
    pub next: i32,         // = -1;
}
