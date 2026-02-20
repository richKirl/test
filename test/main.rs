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
    // let mut stack = Stack::new(); //стек

    println!("{:.2}", 0.0); // Настройка точности вывода (аналог setprecision)
    println!("=== ИНИЦИАЛИЗАЦИЯ МИРА ===");

    const LAYER_NONE: i32 = 0;
    const LAYER_STATIC: i32 = 1;
    const LAYER_TRIGGER: i32 = 2;
    const LAYER_PLAYER: i32 = 4;

    // 1. Создаем статичную стену
    let wall_id = world.create_entity(
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(1.0, 10.0, 10.0),
        LAYER_STATIC,
        LAYER_NONE,
    );
    println!("[World]: Создана стена ID: {} на x=10", wall_id);

    // 2. Создаем ядовитую зону (Trigger)
    let poison_zone = world.create_entity(
        Vec3::new(5.0, 0.0, 0.0),
        Vec3::new(2.0, 2.0, 2.0),
        LAYER_TRIGGER,
        LAYER_NONE,
    );
    if let Some(e) = world.registry.get_mut(&poison_zone) {
        e.on_trigger = Some(Box::new(|visitor_id| {
            println!(
                "  [EVENT]: Объект {} вступил в ЯДОВИТУЮ ЗОНУ! -10 HP",
                visitor_id
            );
        }));
    }

    // 3. Создаем рычаг (Trigger)
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
    println!("[World]: Игрок создан ID: {}\n", player_id);

    // --- ТЕСТ 1: ДВИЖЕНИЕ СКВОЗЬ ТРИГГЕР ---
    println!("=== ТЕСТ 1: ДВИЖЕНИЕ СКВОЗЬ ТРИГГЕР ===");
    for x in [0.0, 2.0, 4.0, 6.0] {
        world.update_position(player_id, Vec3::new(x, 0.0, 0.0));
        println!("Игрок переместился в x={:.1}", x);

        let player_aabb = world.registry[&player_id].get_aabb();
        let mut nearby = Vec::new();
        world.query(&player_aabb, &mut nearby);

        for id in nearby {
            if id != player_id {
                // Вызываем триггер, если он есть
                if let Some(entity) = world.registry.get_mut(&id) {
                    if let Some(ref mut callback) = entity.on_trigger {
                        callback(player_id);
                    }
                }
            }
        }
    }

    // --- ТЕСТ 2: ВЗАИМОДЕЙСТВИЕ (RAYCAST) ---
    println!("\n=== ТЕСТ 2: ВЗАИМОДЕЙСТВИЕ (RAYCAST) ===");
    let camera_pos = Vec3::new(7.0, 0.0, 2.0); // Встали рядом с рычагом
    let look_dir = Vec3::new(1.0, 0.0, 0.0); // Смотрим в сторону рычага

    println!("Игрок смотрит вперед из точки (7, 0, 2)...");
    let hit = world.raycast(camera_pos, camera_pos + look_dir * 5.0, LAYER_TRIGGER);

    if hit != -1 {
        if let Some(entity) = world.registry.get(&hit) {
            if let Some(ref callback) = entity.on_interact {
                println!("Найден объект для взаимодействия: {}", hit);
                callback();
            }
        }
    }

    // --- ТЕСТ 3: СТОЛКНОВЕНИЕ СО СТЕНОЙ ---
    println!("\n=== ТЕСТ 3: КОЛЛИЗИЯ СО СТЕНОЙ ===");
    let player_pos = world.registry[&player_id].pos;
    let next_pos = Vec3::new(10.0, 0.0, 0.0); // Пытаемся зайти в стену

    let wall_hit = world.raycast(player_pos, next_pos, LAYER_STATIC);

    if wall_hit != -1 {
        println!(
            "[BLOCK]: Движение заблокировано! Впереди стена ID: {}",
            wall_hit
        );
    }

    // --- ТЕСТ 4: Очистка ---
    println!("\n=== ТЕСТ 4: Очистка ===");
    // В Rust мы просто помечаем всё на удаление или делаем полный сброс
    let ids: Vec<i32> = world.registry.keys().cloned().collect();
    for id in ids {
        world.mark_for_deletion(id);
    }
    world.cleanup();

    println!("[LOG]: All entities removed. Root: {}", world.bvh.root);
    println!("\n=== ВСЕ ТЕСТЫ ЗАВЕРШЕНЫ ===");
}
