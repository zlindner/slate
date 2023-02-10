use crate::prisma::character;

pub struct Character {
    pub id: i32,
    pub map_id: i32,
    pub data: character::Data,
    pub position: (i32, i32),
    pub stance: i32,
}

impl Character {
    pub fn new(data: character::Data) -> Self {
        Self {
            id: data.id,
            map_id: data.map,
            data,
            position: (0, 0),
            stance: 0,
        }
    }
}
