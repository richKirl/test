use crate::Aabb;
use crate::DynamicBvh;
use crate::Stack;
use crate::Vec3;
use crate::entity::Entity;
use std::collections::HashMap;
pub struct World {
    pub bvh: DynamicBvh,
    pub registry: HashMap<i32, Entity>, //std::map<int, std::unique_ptr<Entity>> registry;
    pub entity_to_node: HashMap<i32, i32>, //std::map<int, int> entityToNode;
    pub next_id: i32,                   //int nextId = 0;
    pub que_delete: Vec<i32>,           //std::vector<int> deletionQueue;
}
#[rustfmt::skip]
impl World {
    pub fn new() -> Self {
        Self {
            bvh: DynamicBvh {
                nodes: Vec::new(),
                root: -1,
                free_list: -1,
                margin: 0.2,
            },
            registry: HashMap::new(),
            entity_to_node: HashMap::new(),
            next_id: 0,
            que_delete: Vec::new(),
        }
    }
    pub fn create_entity(&mut self, pos: Vec3, size: Vec3, cat: i32, mask: i32) -> i32 {
        self.next_id += 1;
        let id = self.next_id;
        let e = Entity::new(id, pos, size, cat, mask);
        self.entity_to_node
            .insert(id, self.bvh.insert_leaf(id, &e.get_aabb()));
        self.registry.insert(id, e);
        return id;
    }
    pub fn update_position(&mut self, id: i32, npos: Vec3) {
        if let Some(entity) = self.registry.get_mut(&id) {
            entity.pos = npos;
            let real_aabb = entity.get_aabb();

            // 1. Получаем текущий индекс узла (копируем его, чтобы отпустить ссылку на hashmap)
            let old_node_idx = *self.entity_to_node.get(&id).expect("Entity not in BVH");

            // 2. Проверяем, нужно ли обновление
            if !self.bvh.nodes[old_node_idx as usize]
                .bbox
                .contains(real_aabb)
            {
                // Удаляем старый
                self.bvh.remove_leaf(old_node_idx);

                // Вставляем новый (с запасом margin!)
                let fat_aabb = Aabb::new(
                    real_aabb.min - Vec3::splat(self.bvh.margin),
                    real_aabb.max + Vec3::splat(self.bvh.margin),
                );
                let new_node_idx = self.bvh.insert_leaf(id, &fat_aabb);

                // Обновляем мапу
                self.entity_to_node.insert(id, new_node_idx);
            }
        }
    }
    pub fn query(&self, bbox: &Aabb, out: &mut Vec<i32>) {
        if self.bvh.root == -1 { return; }

        let mut stack = Stack::new();
        stack.push(self.bvh.root);

        while !stack.is_empty() {

            let mut node_idx = 0;
            if let Some(p) = stack.pop(){
                node_idx=p;
            };
            let node = &self.bvh.nodes[node_idx as usize];

            // Проверка на пересечение (Intersects), а не на удержание (Contains)
            if node.bbox.min.x > bbox.max.x || node.bbox.max.x < bbox.min.x ||
               node.bbox.min.y > bbox.max.y || node.bbox.max.y < bbox.min.y ||
               node.bbox.min.z > bbox.max.z || node.bbox.max.z < bbox.min.z {
                continue;
            }

            if node.is_leaf {
                out.push(node.object_index);
            } else {
                // Всегда проверяйте переполнение стека, если дерево глубокое
                stack.push(node.child1);
                stack.push(node.child2);
            }
        }
    }

    pub fn raycast(p1:Vec3,p2:Vec3,mask:i32) -> i32 {
        mask
    }

    pub fn mark_for_deletion(&mut self, id: i32) {
        if let Some(entity) = self.registry.get_mut(&id) {
            if !entity.gameplay.is_dirty {
                entity.gameplay.is_dirty = true;
                self.que_delete.push(id);
            }
        }
    }

    pub fn cleanup(&mut self) {
        // Используем drain, чтобы очистить очередь и получить ID
        for id in self.que_delete.drain(..) {
            // 1. Удаляем из BVH
            if let Some(node_idx) = self.entity_to_node.remove(&id) {
                self.bvh.remove_leaf(node_idx);
            }
            // 2. Удаляем саму сущность
            self.registry.remove(&id);
        }
    }
    pub fn clear_all(&mut self) {
        self.cleanup();
        self.registry.clear();
        self.entity_to_node.clear();
        self.que_delete.clear();
        self.next_id = 0;
        // Сброс самого BVH
        self.bvh.nodes.clear();
        self.bvh.root = -1;
        self.bvh.free_list = -1;
    }
}
