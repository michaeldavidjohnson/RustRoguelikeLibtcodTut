use tcod::colors::*;
use tcod::console::*;
use tcod::input::Key;
use tcod::input::KeyCode::*;

const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;
const FPS: i32 = 25;

struct Tcod {
    root: Root,
}

fn handle_keys(tcod: &mut Tcod, player_x: &mut i32, player_y: &mut i32) -> bool {
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
        } => *player_y -= 1,
        Key {
            code: tcod::input::KeyCode::Down,
            ..
        } => *player_y += 1,
        Key {
            code: tcod::input::KeyCode::Left,
            ..
        } => *player_x -= 1,
        Key {
            code: tcod::input::KeyCode::Right,
            ..
        } => *player_x += 1,
        _ => {}
    }
    return false;
}

fn main() {
    let root = Root::initializer()
        .font("./resources/arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("Tutorial")
        .init();

    let mut tcod = Tcod { root };
    tcod::system::set_fps(FPS);

    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT / 2;

    while !tcod.root.window_closed() {
        tcod.root.set_default_foreground(WHITE);
        tcod.root.clear();
        tcod.root
            .put_char(player_x, player_y, '@', BackgroundFlag::None);
        tcod.root.flush();

        let exit = handle_keys(&mut tcod, &mut player_x, &mut player_y);
        if exit {
            break;
        }
    }
    println!("Hello, world!");
}
