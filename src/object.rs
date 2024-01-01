use tcod::{colors::WHITE, Color, Console};

use crate::{
    components,
    game::{self, Game},
    is_blocked, Map,
};

#[derive(Debug)]
pub struct Object {
    pub x: i32,
    pub y: i32,
    pub char: char,
    pub color: Color,
    pub name: String,
    pub blocks_motion: bool,
    pub is_alive: bool,
    pub fighter: Option<components::Fighter>,
    pub ai: Option<components::Ai>,
    pub item: Option<components::Item>,
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
            fighter: None,
            ai: None,
            item: None,
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

    pub fn distance_to(&self, other: &Object) -> f32 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        ((dx.pow(2) + dy.pow(2)) as f32).sqrt()
    }

    pub fn take_damage(&mut self, damage: i32, game: &mut Game) {
        if let Some(fighter) = self.fighter.as_mut() {
            if damage > 0 {
                fighter.hp -= damage;
            }
        };
        if let Some(fighter) = self.fighter {
            if fighter.hp <= 0 {
                self.is_alive = false;
                fighter.on_death.callback(self, game)
            }
        }
    }

    pub fn attack(&mut self, target: &mut Object, game: &mut Game) {
        let damage = self.fighter.map_or(0, |f| f.power) - target.fighter.map_or(0, |f| f.defense);
        if damage > 0 {
            game.messages.add(
                format!(
                    "{} attacks {} for {} hit points.",
                    self.name, target.name, damage
                ),
                WHITE,
            );
            target.take_damage(damage, game);
        } else {
            game.messages.add(
                format!(
                    "{} attacks {} but it has no effect!",
                    self.name, target.name
                ),
                WHITE,
            );
        }
    }
}
