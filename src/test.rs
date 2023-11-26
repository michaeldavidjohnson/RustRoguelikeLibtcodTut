use crate::{make_empty_map, tile::Tile};



#[test]
fn wall_set_test() {
    let mut map = make_empty_map();
    map[30][22] = Tile::wall();

    assert!(map[30][22].blocked == true)
}
