use crate::Aabb;
use crate::Vec3;
pub struct EntityData {
    pub health: f32,
    pub is_dirty: bool,
}
#[repr(C)]
pub struct Entity {
    pub id: i32,
    pub pos: Vec3,
    pub size: Vec3,
    pub category: i32,
    pub mask: i32,
    pub gameplay: EntityData,
    pub on_trigger: Option<Box<dyn FnMut(i32)>>,
    pub on_interact: Option<Box<dyn Fn()>>,
}
impl Entity {
    pub fn new(id: i32, pos: Vec3, size: Vec3, cat: i32, mask: i32) -> Self {
        Self {
            id: id,
            pos: pos,
            size: size,
            category: cat,
            mask: mask,
            gameplay: EntityData {
                health: 100.0,
                is_dirty: false,
            },
            on_interact: None,
            on_trigger: None,
        }
    }
    pub fn get_aabb(&self) -> Aabb {
        return Aabb {
            min: self.pos - (self.size * 0.5),
            max: self.pos + (self.size * 0.5),
        };
    }
}
