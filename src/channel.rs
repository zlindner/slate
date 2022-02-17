pub struct Channel {
    pub id: i32,
    world_id: i32,
}

impl Channel {
    pub fn new(id: i32, world_id: i32) -> Self {
        Channel { id, world_id }
    }
}
