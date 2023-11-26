mod game;
mod object;
mod roomgen;
mod test;
mod tile;
use std::cmp;

use game::Game;
use object::Object;
use rand::Rng;
use roomgen::Rect;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;

use tile::Tile;

const SCREEN_WIDTH: i32 = 80;
type Map = Vec<Vec<tile::Tile>>;
const SCREEN_HEIGHT: i32 = 50;
const MAP_WIDTH: i32 = 80;
const MAP_HEIGHT: i32 = 45;
const COLOR_DARK_WALL: Color = Color { r: 0, g: 0, b: 100 };
const COLOR_DARK_GROUND: Color = Color {
    r: 50,
    g: 50,
    b: 150,
};
const FPS: i32 = 25;
const ROOM_MAX_SIZE: i32 = 10;
const ROOM_MIN_SIZE: i32 = 6;
const MAX_ROOMS: i32 = 30;

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn make_map(player: &mut object::Object) -> Map {
    let mut map = vec![vec![Tile::wall(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
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
            if rooms.is_empty() {
                player.x = new_x;
                player.y = new_y;
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

fn render_all(tcod: &mut Tcod, game: &Game, objects: &[Object]) {
    for object in objects {
        object.draw(&mut tcod.con);
    }

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            let wall = game.map[x as usize][y as usize].block_sight;
            if wall {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_WALL, BackgroundFlag::Set);
            } else {
                tcod.con
                    .set_char_background(x, y, COLOR_DARK_GROUND, BackgroundFlag::Set)
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
}

fn handle_keys(tcod: &mut Tcod, player: &mut object::Object, game: &Game) -> bool {
    let key = tcod.root.wait_for_keypress(true);
    match key {
        Key {
            code: tcod::input::KeyCode::Enter,
            alt: true,
            ..
        } => {
            let fullscreen = tcod.root.is_fullscreen();
            tcod.root.set_fullscreen(!fullscreen);
        }
        Key {
            code: tcod::input::KeyCode::Escape,
            ..
        } => return true,
        Key {
            code: tcod::input::KeyCode::Up,
            ..
        } => player.move_by(0, -1, game),
        Key {
            code: tcod::input::KeyCode::Down,
            ..
        } => player.move_by(0, 1, game),
        Key {
            code: tcod::input::KeyCode::Left,
            ..
        } => player.move_by(-1, 0, game),
        Key {
            code: tcod::input::KeyCode::Right,
            ..
        } => player.move_by(1, 0, game),
        _ => {}
    }
    return false;
}

fn main() {
    //map[30][22] = Tile::wall();
    //map[33][24] = Tile::wall();

    let root = Root::initializer()
        .font("./resources/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Tutorial")
        .init();

    let mut player = object::Object::new(25, 23, '@', WHITE);
    let map = make_map(&mut player);
    let game = Game { map: map };

    let npc = object::Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);
    let mut objects = [player, npc];

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(FPS);

    let _player_x = SCREEN_WIDTH / 2;
    let _player_y = SCREEN_HEIGHT / 2;

    while !tcod.root.window_closed() {
        tcod.con.clear();
        render_all(&mut tcod, &game, &objects);

        tcod.root.flush();
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player, &game);
        if exit {
            break;
        }
    }
    println!("Hello, world!");
}
