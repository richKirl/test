use crate::Aabb;
use crate::Stack;
use crate::node::Node;
use crate::ray::Ray;
pub struct DynamicBvh {
    pub nodes: Vec<Node>,
    pub root: i32,
    pub free_list: i32, // Индекс первого свободного узла для переиспользования
    pub margin: f32,    // = 0.2f;
}
#[rustfmt::skip]
impl DynamicBvh {
    pub fn allocate_node(&mut self) -> i32 {
        if self.free_list == -1 {
            let idx = self.nodes.len() as i32;
            self.nodes.push(Node::default());
            idx
        } else {
            let idx = self.free_list;
            self.free_list = self.nodes[idx as usize].next;
            idx
        }
    }
    pub fn free_node(&mut self, index: i32) {
        self.nodes[index as usize].next = self.free_list;
        self.free_list = index;
    }
    // Вспомогательная функция для расчета стоимости
    fn calc_entry_cost(&self, node_idx: i32, leaf_bbox: &Aabb, leaf_area: f32) -> f32 {
        let node = &self.nodes[node_idx as usize];
        let combined = Aabb::union(&node.bbox, &leaf_bbox);
        if node.is_leaf {
            Aabb::area(&combined) + leaf_area
        } else {
            Aabb::area(&combined) - Aabb::area(&node.bbox)
        }
    }
    pub fn insert_leaf(&mut self, obj_idx: i32, bbox: &Aabb) -> i32 {
        let leaf_idx = self.allocate_node();
        {
            let node = &mut self.nodes[leaf_idx as usize];
            node.bbox = *bbox;
            node.object_index = obj_idx;
            node.is_leaf = true;
            node.height = 0;
            node.parent_index = -1;
        }

        if self.root == -1 { self.root = leaf_idx; return leaf_idx; }
        let mut index = self.root;
        let leaf_area = Aabb::area(&bbox);

        while !self.nodes[index as usize].is_leaf {
            let node = &self.nodes[index as usize];

            let area = Aabb::area(&node.bbox);
            let combined_area = Aabb::area(&Aabb::union(&node.bbox, bbox));

            // Стоимость создания нового родителя здесь
            let cost = 2.0 * combined_area;
            let inheritance_cost = 2.0 * (combined_area - area);

            // Стоимость спуска в левого или правого потомка
            let cost_left = self.calc_entry_cost(node.child1, bbox, leaf_area) + inheritance_cost;
            let cost_right = self.calc_entry_cost(node.child2, bbox, leaf_area) + inheritance_cost;

            // Нашли лучшее место
            if cost < cost_left && cost < cost_right { break; }

            index = if cost_left < cost_right { node.child1 } else { node.child2 };
        }
        // 2. Создание нового родителя и пересборка иерархии
        let old_parent = self.nodes[index as usize].parent_index;
        let new_parent = self.allocate_node();

        // 1. Настройка нового родителя
        {
            let nodes_ptr = self.nodes.as_mut_ptr();
            unsafe {
                let np = &mut *nodes_ptr.add(new_parent as usize);
                let old_idx_node = &*nodes_ptr.add(index as usize);

                np.parent_index = old_parent;
                np.bbox = Aabb::union(&old_idx_node.bbox, bbox);
                np.child1 = index;
                np.child2 = leaf_idx;
                np.height = old_idx_node.height + 1;
                np.is_leaf = false;
            }
        }

        // 2. Обновляем родителя у существующих узлов
        self.nodes[index as usize].parent_index = new_parent;
        self.nodes[leaf_idx as usize].parent_index = new_parent;

        // 3. Подключаем новый родитель к дереву (выше по иерархии)
        if old_parent != -1 {
            let p = &mut self.nodes[old_parent as usize];
            if p.child1 == index { p.child1 = new_parent; } else { p.child2 = new_parent; }
        } else { self.root = new_parent; }

        // 4. Проход вверх для обновления BBox и балансировки
        self.sync_hierarchie(leaf_idx);

        leaf_idx
    }
    pub fn remove_leaf(&mut self, index: i32) {
        if index == self.root {
            self.root = -1;
            self.free_node(index);
            return;
        }

        let p = self.nodes[index as usize].parent_index;
        let gp = self.nodes[p as usize].parent_index; // Исправлено: берем родителя родителя

        let sib = if self.nodes[p as usize].child1 == index {
            self.nodes[p as usize].child2
        } else {
            self.nodes[p as usize].child1
        };

        if gp != -1 {
            // Подключаем брата (sib) напрямую к дедушке (gp)
            if self.nodes[gp as usize].child1 == p {
                self.nodes[gp as usize].child2 = sib; // Ошибка была здесь: должен быть тот же слот
                self.nodes[gp as usize].child1 = sib;
            } else {
                self.nodes[gp as usize].child2 = sib;
            }
            self.nodes[sib as usize].parent_index = gp;
            self.free_node(p);
            self.sync_hierarchie(sib); // Обновляем дерево начиная с выжившего брата
        } else {
            self.root = sib;
            self.nodes[sib as usize].parent_index = -1;
            self.free_node(p);
        }
        self.free_node(index);
    }

    pub fn sync_hierarchie(&mut self, index: i32) {
        let mut curr = self.nodes[index as usize].parent_index;
        while curr != -1 {
            // Сначала балансируем узел, получаем новый индекс (если был поворот)
            curr = self.balance(curr);

            // Обновляем данные именно в curr!
            self.update_node(curr);

            // Двигаемся выше
            curr = self.nodes[curr as usize].parent_index;
        }
    }
    pub fn update_node(& mut self, index: i32) {
        let c1 = self.nodes[index as usize].child1;
        let c2 = self.nodes[index as usize].child2;
        self.nodes[index as usize].bbox = Aabb::union(&self.nodes[c1 as usize].bbox, &self.nodes[c2 as usize].bbox);
        self.nodes[index as usize].height = 1 + i32::max(self.nodes[c1 as usize].height,self.nodes[c2 as usize].height,
        );
    }
    pub fn balance(&mut self, index: i32) -> i32 {
        if self.nodes[index as usize].is_leaf || self.nodes[index as usize].height < 2 {return index;}

        let c1 = self.nodes[index as usize].child1;
        let c2 = self.nodes[index as usize].child2;
        let balance = self.nodes[c2 as usize].height - self.nodes[c1 as usize].height;

        if balance > 1{
            let r = c2;
            let rl=self.nodes[r as usize].child1;
            let rr=self.nodes[r as usize].child2;
            self.nodes[r as usize].child1 = index;
            self.nodes[r as usize].parent_index = self.nodes[index as usize].parent_index;
            self.nodes[index as usize].parent_index = r;
            if self.nodes[r as usize].parent_index !=-1{
                let parent_idx = self.nodes[r as usize].parent_index;
                if self.nodes[parent_idx as usize].child1 == index {
                    self.nodes[parent_idx as usize].child1 = r;
                }
                else {
                    self.nodes[parent_idx as usize].child2 = r;
                }
            }
            else{ self.root = r;}
            if self.nodes[rl as usize].height > self.nodes[rr as usize].height{
                self.nodes[r as usize].child2 = rl;
                self.nodes[index as usize].child2 = rr;
                self.nodes[rr as usize].parent_index = index;
            }
            else {
                self.nodes[r as usize].child2 = rr;
                self.nodes[index as usize].child2 = rl;
                self.nodes[rr as usize].parent_index = index;
            }
            self.update_node(index);
            self.update_node(r);
            return r;
        }
        if balance < -1 {
            let l = c1;
            let ll=self.nodes[l as usize].child1;
            let lr=self.nodes[l as usize].child2;
            self.nodes[l as usize].child1 = index;
            self.nodes[l as usize].parent_index = self.nodes[index as usize].parent_index;
            if self.nodes[l as usize].parent_index !=-1{
                let parent_idx = self.nodes[l as usize].parent_index;
                if self.nodes[parent_idx as usize].child1 == index {
                    self.nodes[parent_idx as usize].child1 = l;
                }
                else {
                    self.nodes[parent_idx as usize].child2 = l;
                }
            }
            else{ self.root = l;}
            if self.nodes[ll as usize].height > self.nodes[lr as usize].height{
                self.nodes[l as usize].child2 = ll;
                self.nodes[index as usize].child1 = lr;
                self.nodes[lr as usize].parent_index = index;
            }
            else {
                self.nodes[l as usize].child2 = lr;
                self.nodes[index as usize].child1 = ll;
                self.nodes[lr as usize].parent_index = index;
            }
            self.update_node(index);
            self.update_node(l);
            return l;
        }

        index
    }

    pub fn ray_cast(&self, ray: &Ray) -> Vec<i32> {
        let mut results = Vec::new();
        if self.root == -1 { return results; }

        let mut stack= Stack::new();
        stack.push(self.root);

        while !stack.is_empty() {
            let mut node_idx = 0;
            if let Some(p)=stack.pop(){
                node_idx=p;
            };
            let node = &self.nodes[node_idx as usize];

            // Проверяем, пересекает ли луч текущий AABB (узел или лист)
            if node.bbox.intersect_ray(ray) {
                if node.is_leaf {
                    results.push(node.object_index);
                } else {
                    // Добавляем детей в стек для дальнейшей проверки
                    // (для оптимизации можно сначала класть того, кто ближе к лучу)
                    stack.push(node.child1);
                    stack.push(node.child2);
                }
            }
        }
        results
    }
}
