mod game;
mod object;
mod test;
mod tile;
use game::Game;
use object::Object;
use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;
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

struct Tcod {
    root: Root,
    con: Offscreen,
}

fn make_map() -> Map {
    let mut map = vec![vec![Tile::empty(); MAP_HEIGHT as usize]; MAP_WIDTH as usize];
    map
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
    let mut map = make_map();
    map[30][22] = Tile::wall();
    map[33][24] = Tile::wall();
    let game = Game { map: map };

    let root = Root::initializer()
        .font("./resources/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Tutorial")
        .init();

    let player = object::Object::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2, '@', WHITE);
    let npc = object::Object::new(SCREEN_WIDTH / 2 - 5, SCREEN_HEIGHT / 2, '@', YELLOW);
    let mut objects = [player, npc];

    let con = Offscreen::new(MAP_WIDTH, MAP_HEIGHT);
    let mut tcod = Tcod { root, con };
    tcod::system::set_fps(FPS);

    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT / 2;

    while !tcod.root.window_closed() {
        tcod.con.clear();
        render_all(&mut tcod, &game, &objects);

        tcod.root.flush();
        let player = &mut objects[0];
        let exit = handle_keys(&mut tcod, player,&game);
        if exit {
            break;
        }
    }
    println!("Hello, world!");
}
