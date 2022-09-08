pub struct Item {
    pub id: i32,
    pub position: i16,
    pub item_type: u8, // 1: equip, 2: other, 3: pet
}

impl Item {
    pub fn is_cash(&self) -> bool {
        let item_type = self.id / 100000;

        if item_type == 5 {
            return true;
        }

        if item_type != 1 {
            return false;
        }

        // TODO get equip stats for item id
        // check if equip stats are not null and has cash attr
        false
    }
}
