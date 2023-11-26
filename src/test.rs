

#[test]
fn wall_set_test() {
    let mut map = make_map();
    map[30][22] = Tile::wall();

    assert!(map[30][22].blocked == true)
}
