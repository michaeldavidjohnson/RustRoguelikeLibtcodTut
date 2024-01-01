mod ai;
mod components;
mod game;
mod inventory;
mod messages;
mod object;
mod roomgen;
mod statusbar;
mod test;
mod tile;
use core::num;
use std::cmp;

use components::Item;
use game::Game;
use inventory::inventory_menu;
use object::Object;
use rand::Rng;
use roomgen::Rect;
use tcod::colors;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::{self, Event, Key, Mouse};

use tcod::map::{FovAlgorithm, Map as FovMap};
use tile::Tile;

use crate::components::Fighter;
use crate::messages::Messages;
use crate::statusbar::render_bar;

const MSG_X: i32 = BAR_WIDTH + 2;
const MSG_WIDTH: i32 = SCREEN_WIDTH - BAR_WIDTH - 2;
const MSG_HEIGHT: usize = PANEL_HEIGHT as usize - 1;
const SCREEN_WIDTH: i32 = 80;
const BAR_WIDTH: i32 = 20;
const PANEL_HEIGHT: i32 = 7;
const PANEL_Y: i32 = SCREEN_HEIGHT - PANEL_HEIGHT;
type Map = Vec<Vec<tile::Tile>>;
const MAX_ROOM_MONSTERS: i32 = 4;
const MAX_ROOM_ITEMS: i32 = 3;
const INVENTORY_WIDTH: i32 = 50;
const SCREEN_HEIGHT: i32 = 50;
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 43;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const COLOR_LIGHT_WALL: Color = Color {
    r: 130,
    g: 110,
    b: 50,
};
const COLOR_LIGHT_GROUND: Color = Color {
    r: 200,
    g: 180,
    b: 80,
};
const FPS: i32 = 25;
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;
const FOV_ALGORITHM: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;
const TORCH_RADIUS: i32 = 10;

struct Tcod {
    root: Root,
    con: Offscreen,
    panel: Offscreen,
    fov: FovMap,
    key: Key,
    mouse: Mouse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum PlayerAction {
    TookTurn,
    DidntTakeTurn,
    Exit,
}

pub fn is_blocked(x: i32, y: i32, map: &Map, objects: &Vec<Object>) -> bool {
    if map[x as usize][y as usize].blocked {
        return true;
    };
    objects
        .iter()
        .any(|object| object.blocks_motion && object.pos() == (x, y))
}

/// Mutably borrow two *separate* elements from the given slice.
/// Panics when the indexes are equal or out of bounds.
fn mut_two<T>(first_index: usize, second_index: usize, items: &mut [T]) -> (&mut T, &mut T) {
    assert!(first_index != second_index);
    let split_at_index = cmp::max(first_index, second_index);
    let (first_slice, second_slice) = items.split_at_mut(split_at_index);
    if first_index < second_index {
        (&mut first_slice[first_index], &mut second_slice[0])
    } else {
        (&mut second_slice[0], &mut first_slice[second_index])
    }
}

fn move_towards(id: usize, target_x: i32, target_y: i32, game: &Game, objects: &mut Vec<Object>) {
    let dx = target_x - objects[id].x;
    let dy = target_y - objects[id].y;
    let distance = ((dx.pow(2) + dy.pow(2)) as f32).sqrt();
    let dx = (dx as f32 / distance).round() as i32;
    let dy = (dy as f32 / distance).round() as i32;
    move_by(id, dx, dy, game, objects)
}

pub fn pick_item_up(object_id: usize, game: &mut Game, objects: &mut Vec<Object>) {
    if game.inventory.len() >= 26 {
        game.messages.add(
            format!(
                "The inventory is full, cannot pick up {}.",
                objects[object_id].name
            ),
            RED,
        );
    } else {
        let item = objects.swap_remove(object_id);
        game.messages
            .add(format!("You picked up a {}!", item.name), GREEN);
        game.inventory.push(item);
    }
}

pub fn player_move_or_attack(
    id: usize,
    dx: i32,
    dy: i32,
    game: &mut game::Game,
    objects: &mut Vec<Object>,
) {
    let (mut x, mut y) = objects[id].pos();
    x = x + dx;
    y = y + dy;

    let target_id = objects
        .iter()
        .position(|object| object.pos() == (x, y) && object.fighter.is_some());
    match target_id {
        Some(target_id) => {
            let (player, target) = mut_two(0, target_id, objects);
            player.attack(target, game);
        }
        None => move_by(id, dx, dy, game, objects),
    }
}

fn get_names_under_mouse(mouse: Mouse, objects: &[Object], fov_map: &FovMap) -> String {
    let (x, y) = (mouse.cx as i32, mouse.cy as i32);

    // create a list with the names of all objects at the mouse's coordinates and in FOV
    let names = objects
        .iter()
        .filter(|obj| obj.pos() == (x, y) && fov_map.is_in_fov(obj.x, obj.y))
        .map(|obj| obj.name.clone())
        .collect::<Vec<_>>();

    names.join(", ") // join the names, separated by commas
}

pub fn move_by(id: usize, dx: i32, dy: i32, game: &game::Game, objects: &mut Vec<Object>) {
    let (x, y) = objects[id].pos();
    if !is_blocked(x + dx, y + dy, &game.map, objects) {
        objects[id].set_pos(x + dx, y + dy);
    }
}

fn place_objects(room: &Rect, objects: &mut Vec<Object>, game: &game::Game) {
    let num_monsters = rand::thread_rng().gen_range(0, MAX_ROOM_MONSTERS + 1);
    for _ in 0..num_monsters {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);
        if is_blocked(x, y, &game.map, objects) {
            let mut monster = if rand::random::<f32>() < 0.8 {
                let mut orc = Object::new(
                    x,
                    y,
                    'o',
                    colors::DESATURATED_GREEN,
                    "Orc".to_string(),
                    true,
                    true,
                );
                orc.ai = Some(components::Ai::Basic);
                orc.fighter = Some(Fighter {
                    max_hp: 10,
                    hp: 10,
                    defense: 0,
                    power: 3,
                    on_death: components::DeathCallback::Monster,
                });
                orc
            } else {
                let mut troll = Object::new(
                    x,
                    y,
                    'T',
                    colors::DARKER_GREEN,
                    "Troll".to_string(),
                    true,
                    true,
                ); //troll
                troll.ai = Some(components::Ai::Basic);
                troll.fighter = Some(Fighter {
                    max_hp: 16,
                    hp: 16,
                    defense: 1,
                    power: 4,
                    on_death: components::DeathCallback::Monster,
                });
                troll
            };

            objects.push(monster);
        }
    }

    let num_items = rand::thread_rng().gen_range(0, MAX_ROOM_ITEMS + 1);
    for _ in 0..num_items {
        let x = rand::thread_rng().gen_range(room.x1 + 1, room.x2);
        let y = rand::thread_rng().gen_range(room.y1 + 1, room.y2);

        if is_blocked(x, y, &game.map, objects) {
            let mut object = Object::new(
                x,
                y,
                '!',
                VIOLET,
                "Potion of Healing".to_string(),
                false,
                true,
            );
            object.item = Some(Item::Heal);
            objects.push(object);
        }
    }
}

fn make_empty_map() -> Map {
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    map
}

fn make_map(objects: &mut Vec<Object>) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    let game = Game {
        map: map.clone(),
        messages: Messages::new(),
        inventory: vec![],
    };
    //let room1 = Rect::new(20,15,10,15);
    //let room2 = Rect::new(50,15,10,15);
    //create_room(room1, &mut map);
    //create_room(room2, &mut map);
    //create_h_tunnel(25, 55, 23, &mut map);
    let mut rooms = vec![];

    for _ in 0..MAX_ROOMS {
        let w = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let h = rand::thread_rng().gen_range(ROOM_MIN_SIZE, ROOM_MAX_SIZE + 1);
        let x = rand::thread_rng().gen_range(0, MAP_WIDTH - w);
        let y = rand::thread_rng().gen_range(0, MAP_HEIGHT - h);
        let new_room = Rect::new(x, y, w, h);
        let failed = rooms
            .iter()
            .any(|other_room| new_room.intersects_with(other_room));
        if !failed {
            let (new_x, new_y) = new_room.center();
            create_room(new_room.clone(), &mut map);
            place_objects(&new_room, objects, &game);
            if rooms.is_empty() {
                objects[0].set_pos(new_x, new_y)
            } else {
                let (prev_x, prev_y) = rooms[rooms.len() - 1].center();
                if rand::random() {
                    create_h_tunnel(prev_x, new_x, prev_y, &mut map);
                    create_v_tunnel(prev_y, new_y, new_x, &mut map);
                } else {
                    create_v_tunnel(prev_y, new_y, prev_x, &mut map);
                    create_h_tunnel(prev_x, new_x, new_y, &mut map);
                }
            }
            rooms.push(new_room.clone());
        }
    }

    map
}

fn create_room(room: Rect, map: &mut Map) {
    for x in (room.x1 + 1)..room.x2 {
        for y in (room.y1 + 1)..room.y2 {
            map[x as usize][y as usize] = Tile::empty();
        }
    }
}

fn create_h_tunnel(x1: i32, x2: i32, y: i32, map: &mut Map) {
    for x in cmp::min(x1, x2)..(cmp::max(x1, x2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn create_v_tunnel(y1: i32, y2: i32, x: i32, map: &mut Map) {
    for y in cmp::min(y1, y2)..(cmp::max(y1, y2) + 1) {
        map[x as usize][y as usize] = Tile::empty();
    }
}

fn render_all(tcod: &mut Tcod, game: &mut Game, objects: &[Object], fov_recompute: bool) {
    if fov_recompute {
        let player = &objects[0];
        tcod.fov.compute_fov(
            player.x,
            player.y,
            TORCH_RADIUS,
            FOV_LIGHT_WALLS,
            FOV_ALGORITHM,
        );
    }

    let mut to_draw: Vec<_> = objects
        .iter()
        .filter(|o| tcod.fov.is_in_fov(o.x, o.y))
        .collect();
    to_draw.sort_by(|o1, o2| o1.blocks_motion.cmp(&o2.blocks_motion));
    for object in to_draw {
        if tcod.fov.is_in_fov(object.x, object.y) {
            object.draw(&mut tcod.con);
        }
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let visible = tcod.fov.is_in_fov(x, y);
            let wall = game.map[x as usize][y as usize].block_sight;
            let colour = match (visible, wall) {
                (false, true) => COLOR_DARK_WALL,
                (false, false) => COLOR_DARK_GROUND,
                (true, true) => COLOR_LIGHT_WALL,
                (true, false) => COLOR_LIGHT_GROUND,
            };
            let explored = &mut game.map[x as usize][y as usize].explored;
            if visible {
                *explored = true;
            }
            if *explored {
                tcod.con
                    .set_char_background(x, y, colour, BackgroundFlag::Set);
            }
        }
    }

    blit(
        &tcod.con,
        (0, 0),
        (MAP_WIDTH, MAP_HEIGHT),
        &mut tcod.root,
        (0, 0),
        1.0,
        1.0,
    );

    tcod.panel.set_default_background(BLACK);
    tcod.panel.clear();

    let hp = objects[0].fighter.map_or(0, |f| f.hp);
    let max_hp = objects[0].fighter.map_or(0, |f| f.max_hp);
    render_bar(
        &mut tcod.panel,
        1,
        1,
        BAR_WIDTH,
        "HP",
        hp,
        max_hp,
        LIGHT_RED,
        DARKER_RED,
    );

    tcod.panel.set_default_foreground(LIGHT_GREY);
    tcod.panel.print_ex(
        1,
        0,
        BackgroundFlag::None,
        TextAlignment::Left,
        get_names_under_mouse(tcod.mouse, objects, &tcod.fov),
    );

    let mut y = MSG_HEIGHT as i32;
    for &(ref msg, color) in game.messages.iter().rev() {
        let msg_height = tcod.panel.get_height_rect(MSG_X, y, MSG_WIDTH, 0, msg);
        y -= msg_height;
        if y < 0 {
            break;
        }
        tcod.panel.set_default_foreground(color);
        tcod.panel.print_rect(MSG_X, y, MSG_WIDTH, 0, msg);
    }

    blit(
        &tcod.panel,
        (0, 0),
        (SCREEN_WIDTH, PANEL_HEIGHT),
        &mut tcod.root,
        (0, PANEL_Y),
        1.0,
        1.0,
    )
}

fn handle_keys(
    tcod: &mut Tcod,
    player_id: usize,
    objects: &mut Vec<Object>,
    game: &mut Game,
) -> PlayerAction {
    let player_alive = objects[player_id].is_alive;
    match (tcod.key, tcod.key.text(), player_alive) {
        (
            Key {
                code: tcod::input::KeyCode::Enter,
                alt: true,
                ..
            },
            _,
            _,
        ) => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
            return PlayerAction::DidntTakeTurn;
        }
        (
            Key {
                code: tcod::input::KeyCode::Escape,
                ..
            },
            _,
            _,
        ) => return PlayerAction::Exit,
        (
            Key {
                code: tcod::input::KeyCode::Up,
                ..
            },
            _,
            true,
        ) => {
            player_move_or_attack(player_id, 0, -1, game, objects);
            return PlayerAction::TookTurn;
        }
        (
            Key {
                code: tcod::input::KeyCode::Down,
                ..
            },
            _,
            true,
        ) => {
            player_move_or_attack(player_id, 0, 1, game, objects);
            return PlayerAction::TookTurn;
        }
        (
            Key {
                code: tcod::input::KeyCode::Left,
                ..
            },
            _,
            true,
        ) => {
            player_move_or_attack(player_id, -1, 0, game, objects);
            return PlayerAction::TookTurn;
        }
        (
            Key {
                code: tcod::input::KeyCode::Right,
                ..
            },
            _,
            true,
        ) => {
            player_move_or_attack(player_id, 1, 0, game, objects);
            return PlayerAction::TookTurn;
        }
        (
            Key {
                code: tcod::input::KeyCode::Text,
                ..
            },
            "g",
            true,
        ) => {
            let item_id = objects
                .iter()
                .position(|object| object.pos() == objects[0].pos() && object.item.is_some());
            if let Some(item_id) = item_id {
                pick_item_up(item_id, game, objects);
            }
            return PlayerAction::DidntTakeTurn;
        }
        (
            Key {
                code: tcod::input::KeyCode::Text,
                ..
            },
            "i",
            true,
        ) => {
            // show the inventory
            inventory_menu(
                &game.inventory,
                "Press the key next to an item to use it, or any other to cancel.\n",
                &mut tcod.root,
            );
            return PlayerAction::TookTurn;
        }

        _ => {
            return PlayerAction::DidntTakeTurn;
        }
    }
}

fn main() {
    //map[30][22] = Tile::wall();
    //map[33][24] = Tile::wall();

    let mut root = Root::initializer()
        .font("./resources/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Tutorial")
        .init();

    let mut player = object::Object::new(25, 23, '@', WHITE, "me".to_string(), true, true);
    player.fighter = Some(Fighter {
        max_hp: 30,
        hp: 30,
        defense: 2,
        power: 5,
        on_death: components::DeathCallback::Player,
    });
    let mut objects = vec![player];
    let map = make_map(&mut objects);
    let mut game = Game {
        map: map,
        messages: Messages::new(),
        inventory: vec![],
    };

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let panel = Offscreen::new(SCREEN_WIDTH, PANEL_HEIGHT);
    let fov = FovMap::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod {
        root: root,
        con: con,
        panel: panel,
        fov: fov,
        mouse: Default::default(),
        key: Default::default(),
    };
    tcod::system::set_fps(FPS);

    let _player_x = SCREEN_WIDTH / 2;
    let _player_y = SCREEN_HEIGHT / 2;

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            tcod.fov.set(
                x,
                y,
                !game.map[x as usize][y as usize].block_sight,
                !game.map[x as usize][y as usize].blocked,
            );
        }
    }

    let mut previous_player_position = (-1, -1);
    game.messages.add("Testing!", RED);
    while !tcod.root.window_closed() {
        tcod.con.clear();
        let fov_recompute = previous_player_position != (objects[0].x, objects[0].y);
        match input::check_for_event(input::MOUSE | input::KEY_PRESS) {
            Some((_, Event::Mouse(m))) => tcod.mouse = m,
            Some((_, Event::Key(k))) => tcod.key = k,
            _ => tcod.key = Default::default(),
        }
        render_all(&mut tcod, &mut game, &objects, fov_recompute);

        tcod.root.flush();
        let player = &mut objects[0];
        previous_player_position = (player.x, player.y);
        let exit = handle_keys(&mut tcod, 0, &mut objects, &mut game);

        if objects[0].is_alive && exit != PlayerAction::DidntTakeTurn {
            for id in 0..objects.len() {
                if objects[id].ai.is_some() {
                    ai::ai_take_turn(id, &tcod, &mut game, &mut objects)
                }
            }
        }

        if exit == PlayerAction::Exit {
            break;
        }
    }
    println!("Hello, world!");
}
