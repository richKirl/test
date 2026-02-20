use crate::world::World;
use aabb::Aabb;
use dynbvh::DynamicBvh;
use glam::Vec3;
use stack::Stack;
mod aabb;
mod dynbvh;
mod entity;
mod node;
mod ray;
mod stack;
mod world;

fn main() {
    let mut world = World::new();

    println!("=== ИНИЦИАЛИЗАЦИЯ МИРА ===");

    // Константы слоев (для наглядности)
    const LAYER_NONE: i32 = 0;
    const LAYER_STATIC: i32 = 1;
    const LAYER_TRIGGER: i32 = 2;
    const LAYER_PLAYER: i32 = 4;

    // 1. Создаем стену
    world.create_entity(
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(1.0, 10.0, 10.0),
        LAYER_STATIC,
        LAYER_NONE,
    );

    // 2. Создаем ядовитую зону
    let poison_zone = world.create_entity(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(2.0, 2.0, 2.0),
        LAYER_TRIGGER,
        LAYER_NONE,
    );
    if let Some(e) = world.registry.get_mut(&poison_zone) {
        e.on_trigger = Some(Box::new(|id| {
            println!("  [EVENT]: Объект {} вступил в ЯДОВИТУЮ ЗОНУ!", id);
        }));
    }

    // 3. Создаем рычаг
    let lever_id = world.create_entity(
        Vec3::new(8.0, 0.0, 2.0),
        Vec3::new(0.5, 0.5, 0.5),
        LAYER_TRIGGER,
        LAYER_NONE,
    );
    if let Some(e) = world.registry.get_mut(&lever_id) {
        e.on_interact = Some(Box::new(|| {
            println!("  [INTERACT]: Рычаг нажат! Секретная дверь открыта.");
        }));
    }

    // 4. Создаем игрока
    let player_id = world.create_entity(
        Vec3::new(0.0, 0.0, 0.0),
        Vec3::new(0.6, 1.8, 0.6),
        LAYER_PLAYER,
        LAYER_STATIC | LAYER_TRIGGER,
    );

    println!("\n=== ТЕСТ 1: ДВИЖЕНИЕ СКВОЗЬ ТРИГГЕР ===");
    for x in [0.0, 2.0, 4.0, 6.0] {
        world.update_position(player_id, Vec3::new(x, 0.0, 0.0));
        println!("Игрок переместился в x={:.1}", x);

        let player_aabb = world.registry[&player_id].get_aabb();
        let mut nearby = Vec::new();
        world.query(&player_aabb, &mut nearby);

        for id in nearby {
            if id != player_id {
                // Извлекаем колбэк временно, чтобы избежать проблем с Borrow Checker
                // Или используем Interior Mutability. Здесь проще всего взять ссылку.
                if let Some(entity) = world.registry.get_mut(&id) {
                    if let Some(ref mut callback) = entity.on_trigger {
                        callback(player_id);
                    }
                }
            }
        }
    }

    println!("\n=== ТЕСТ 2: ВЗАИМОДЕЙСТВИЕ ===");
    // Допустим, мы просто ищем объект в точке (Raycast требует отдельной реализации пересечения луча и AABB)
    let interact_pos = Vec3::new(8.0, 0.0, 2.0);
    let mut interact_results = Vec::new();
    world.query(
        &Aabb::new(interact_pos - 0.1, interact_pos + 0.1),
        &mut interact_results,
    );

    for id in interact_results {
        if let Some(entity) = world.registry.get(&id) {
            if let Some(ref callback) = entity.on_interact {
                println!("Найден объект: {}", id);
                callback();
            }
        }
    }

    world.clear_all();
    println!("\n[LOG]: All entities removed. Root: {}", world.bvh.root);
    println!("\n=== ВСЕ ТЕСТЫ ЗАВЕРШЕНЫ ===");
}
