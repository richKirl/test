use crate::Vec3;
use crate::ray::Ray;
#[derive(Default, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
#[rustfmt::skip]
impl Aabb {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self {
            min: min,
            max: max
        }
    }
    pub fn contains(&self,other: Aabb) -> bool{
        self.min.cmple(other.min).all() && self.max.cmpge(other.max).all()
    }
    pub fn merge(&mut self, other: &Aabb) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }
    pub fn union(a: &Aabb, b: &Aabb) -> Aabb {
        Aabb {
            min: a.min.min(b.min),
            max: a.max.max(b.max),
        }
    }
    pub fn area(a:&Aabb) -> f32{
        let d = a.max - a.min;
        return 2.0 * (d.x * d.y + d.y*d.z + d.z*d.x);
    }
    pub fn intersect_ray(&self, ray: &Ray) -> bool {
        let t1 = (self.min - ray.origin) * ray.inv_dir;
        let t2 = (self.max - ray.origin) * ray.inv_dir;

        let t_min = t1.min(t2);
        let t_max = t1.max(t2);

        let t_enter = t_min.max_element();
        let t_exit = t_max.min_element();

        t_exit >= t_enter && t_exit > 0.0
    }
}
