use crate::{game::Game, move_towards, mut_two, object::Object, Tcod};

pub fn ai_take_turn(monster_id: usize, tcod: &Tcod, game: &Game, objects: &mut [Object]) {
    let (monster_x, monster_y) = objects[monster_id].pos();
    if tcod.fov.is_in_fov(monster_x, monster_y) {
        if objects[monster_id].distance_to(&objects[0]) >= 2.0 {
            let (player_x, player_y) = objects[0].pos();
            move_towards(monster_id, player_x, player_y, game, objects)
        } else if objects[0].fighter.map_or(false, |f| f.hp > 0) {
            let (monster, player) = mut_two(monster_id, 0, objects);
            monster.attack(player);
        }
    }
}
