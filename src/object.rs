use tcod::{Color, Console};

use crate::{game, is_blocked, Map};

#[derive(Debug)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub char: char,
    pub color: Color,
    pub name: String,
    pub blocks_motion: bool,
    pub is_alive: bool,
}

impl Object {
    pub fn new(
        x: i32,
        y: i32,
        char: char,
        color: Color,
        name: String,
        blocks_motion: bool,
        is_alive: bool,
    ) -> Self {
        Object {
            x: x,
            y: y,
            char: char,
            color: color,
            name: name,
            blocks_motion: blocks_motion,
            is_alive: is_alive,
        }
    }

    pub fn pos(&self) -> (i32, i32) {
        (self.x, self.y)
    }

    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.x = x;
        self.y = y;
    }

    pub fn draw(&self, con: &mut dyn Console) {
        con.set_default_foreground(self.color);
        con.put_char(self.x, self.y, self.char, tcod::BackgroundFlag::None);
    }
}
