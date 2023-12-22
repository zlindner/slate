use crate::nx::DATA;
use nx::GenericNode;

#[derive(Default)]
pub struct Equipment {
    pub w_atk: Option<i32>,
    pub upgrade_slots: Option<i32>,
}

impl Equipment {
    /// Loads equip data from Character.nx for the given equip id and category
    /// Returns (key, value) pairs
    // FIXME this panics for handaxe?
    // TODO should cache this
    pub fn load(id: i32, equip_type: &EquipmentType) -> Option<Self> {
        let id = format!("0{}.img", id);
        let root = DATA.get("Character").unwrap().root();
        let category = root.get(equip_type.as_str());
        let equip = category.get(&id);

        if equip.is_none() {
            log::debug!("{} of type {} not found", id, equip_type.as_str());
            return None;
        }

        let info = equip.get("info");
        log::debug!("Loading equip data from {}", id);

        let mut nx_equip = Self::default();

        // Weapon attack
        let w_atk = info.get("incPAD");

        if w_atk.is_some() {
            nx_equip.w_atk = Some(w_atk.integer().unwrap() as i32);
        }

        // Upgrade slots
        let upgrade_slots = info.get("tuc");

        if upgrade_slots.is_some() {
            nx_equip.upgrade_slots = Some(upgrade_slots.integer().unwrap() as i32);
        }

        Some(nx_equip)
    }
}

pub enum EquipmentType {
    Bottom,
    Overall,
    Shield,
    Shoes,
    Top,
    Weapon,
}

impl EquipmentType {
    pub fn get_position(&self) -> i32 {
        match self {
            Self::Bottom => 6,
            Self::Overall => todo!(),
            Self::Shield => todo!(),
            Self::Shoes => 7,
            Self::Top => 5,
            Self::Weapon => 11,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bottom => "Pants",
            Self::Overall => "Longcoat",
            Self::Shield => "Shield",
            Self::Shoes => "Shoes",
            Self::Top => "Coat",
            Self::Weapon => "Weapon",
        }
    }
}
