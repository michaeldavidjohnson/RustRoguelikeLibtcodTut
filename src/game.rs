use crate::{messages::Messages, object::Object, Map};

pub struct Game {
    pub map: Map,
    pub messages: Messages,
    pub inventory: Vec<Object>,
}
