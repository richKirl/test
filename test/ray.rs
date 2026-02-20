use glam::Vec3;
pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
    pub inv_dir: Vec3, // 1.0 / direction для скорости
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            inv_dir: Vec3::new(1.0 / direction.x, 1.0 / direction.y, 1.0 / direction.z),
        }
    }
}
