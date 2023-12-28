use super::Portal;
use crate::nx::DATA;
use anyhow::anyhow;
use nx::GenericNode;
use rand::Rng;
use std::collections::HashMap;

pub struct Map {
    pub create_mob_interval: i64,
    pub field_limit: i64,
    pub mob_rate: f64,
    pub on_user_enter: String,
    pub on_first_user_enter: String,
    pub npcs: HashMap<i32, Life>,
    pub monsters: HashMap<i32, Life>,
    pub portals: HashMap<i32, Portal>,
    pub return_map_id: i64,
    pub bounds: (i64, i64, i64, i64),
    pub footholds: Vec<Foothold>,
    pub town: i64,
}

impl Map {
    pub fn load(id: i32) -> anyhow::Result<Self> {
        let root = DATA.get("Map").unwrap().root().get("Map");
        let area_name = format!("Map{}", id / 100000000);
        let area_data = root.get(&area_name);

        if area_data.is_none() {
            return Err(anyhow!("Area data {} not found for map {}", area_name, id));
        }

        let map_name = get_map_img_name(id);
        let map_data = area_data.get(&map_name);

        if map_data.is_none() {
            return Err(anyhow!(
                "Map data {} not found for area {}",
                map_name,
                area_name
            ));
        }

        let map_data = map_data.unwrap();
        let info = map_data.get("info").unwrap();
        log::debug!("Loading map data from {}/{}", area_name, map_name);

        let create_mob_interval = info.get("createMobInterval").integer().unwrap_or(50000);
        let field_limit = info.get("fieldLimit").integer().unwrap_or_default();
        let mob_rate = info.get("mobRate").float().unwrap_or_default();
        let return_map_id = info.get("returnMap").integer().unwrap_or_default();
        let town = info.get("town").integer().unwrap_or_default();

        let mut on_user_enter = info
            .get("onUserEnter")
            .string()
            .unwrap_or_default()
            .to_string();

        if on_user_enter.is_empty() {
            on_user_enter = id.to_string();
        }

        let mut on_first_user_enter = info
            .get("onFirstUserEnter")
            .string()
            .unwrap_or_default()
            .to_string();

        if on_first_user_enter.is_empty() {
            on_first_user_enter = id.to_string();
        }

        let vr_top = info.get("VRTop").integer().unwrap_or_default();
        let vr_bottom = info.get("VRBottom").integer().unwrap_or_default();

        let bounds: (i64, i64, i64, i64) = if vr_top == vr_bottom {
            // Old style baked map: uses point bounds
            let minimap_data = map_data.get("miniMap");

            match minimap_data {
                Some(data) => {
                    let center_x = data.get("centerX").integer().unwrap_or_default();
                    let center_y = data.get("centerY").integer().unwrap_or_default();
                    let width = data.get("width").integer().unwrap_or_default();
                    let height = data.get("height").integer().unwrap_or_default();
                    (-center_x, -center_y, width, height)
                }
                None => {
                    let dist: i64 = 1 << 18;
                    (-dist / 2, -dist / 2, dist, dist)
                }
            }
        } else {
            // Newer(?) map: uses line bounds
            let vr_left = info.get("VRLeft").integer().unwrap_or_default();
            let vr_right = info.get("VRRight").integer().unwrap_or_default();
            (vr_top, vr_bottom, vr_left, vr_right)
        };

        // TODO timeMob
        // TODO area
        // TODO seat
        // TODO add player npc -> playernpcs
        // TODO add player npc -> developer npcs?

        let life_root = map_data.get("life");
        let (npcs, monsters) = match life_root {
            Some(life_root) => Life::load(life_root)?,
            None => (HashMap::new(), HashMap::new()),
        };

        let portal_root = map_data.get("portal");
        let portals = match portal_root {
            Some(portal_root) => Portal::load(portal_root)?,
            None => HashMap::new(),
        };

        let foothold_root = map_data.get("foothold");
        let footholds = match foothold_root {
            Some(foothold_root) => Foothold::load(foothold_root)?,
            None => Vec::new(),
        };

        // TODO load life from db
        // TODO if cpq map load monsterCarnival
        // TODO reactor
        // TODO load map and street name?

        Ok(Self {
            create_mob_interval,
            field_limit,
            mob_rate,
            on_user_enter,
            on_first_user_enter,
            npcs,
            monsters,
            portals,
            return_map_id,
            bounds,
            footholds,
            town,
        })
    }
}

pub struct Life {
    pub id: i32,
    pub life_type: LifeType,
    pub name: String,
    pub position: (i16, i16),
    pub object_id: i32,
    pub stance: i32,
    pub f: u8,
    pub is_hidden: bool,
    pub fh: i16,
    pub start_fh: i16,
    pub cy: i16,
    pub rx0: i16,
    pub rx1: i16,
    pub mob_time: i64,
}

impl Life {
    // TODO rewrite this -- should ideally have load_npcs and load_monsters
    pub fn load(root: nx::Node) -> anyhow::Result<(HashMap<i32, Self>, HashMap<i32, Self>)> {
        let mut npcs = HashMap::new();
        let mut monsters = HashMap::new();

        for life in root.iter() {
            let id = life.get("id").string().unwrap_or_default();
            let type_ = life.get("type").string().unwrap_or_default();

            let life_type = match type_ {
                "n" | "N" => LifeType::NPC,
                "m" | "M" => LifeType::Monster,
                _ => break,
            };

            let team = life.get("team").integer().unwrap_or(-1);
            // TODO team stuff
            let x = life.get("x").integer().unwrap_or(0) as i16;
            let y = life.get("y").integer().unwrap_or(0) as i16;

            let mut life = Life {
                id: id.parse().unwrap_or(0),
                life_type,
                name: String::new(),
                position: (x, y),
                // FIXME this is terrible
                // Maple requires object ids be i32s (🤢) so pick some random number above
                // 10000000 to not collide with character ids. Should probably maintain a set per
                // map that contains used object ids, keep generating until unused.
                object_id: rand::thread_rng().gen_range(10000000..=i32::MAX),
                stance: 0,
                f: life.get("f").integer().unwrap_or(0) as u8,
                is_hidden: life.get("hide").integer().unwrap_or(0) == 1,
                fh: life.get("fh").integer().unwrap_or(0) as i16,
                start_fh: life.get("fh").integer().unwrap_or(0) as i16,
                cy: life.get("cy").integer().unwrap_or(0) as i16,
                rx0: life.get("rx0").integer().unwrap_or(0) as i16,
                rx1: life.get("rx1").integer().unwrap_or(0) as i16,
                mob_time: life.get("mobTime").integer().unwrap_or(0),
            };

            match life.life_type {
                LifeType::NPC => {
                    // TODO wait wtf is the point of this
                    let name = load_npc_string(life.id, "name");
                    life.name = name;

                    npcs.insert(life.object_id, life);
                }
                LifeType::Monster => {
                    // TODO load stats from Mob.nx
                    monsters.insert(life.object_id, life);
                }
            };
        }

        Ok((npcs, monsters))
    }
}

pub enum LifeType {
    NPC,
    Monster,
}

pub struct Foothold {
    pub x1: i64,
    pub y1: i64,
    pub x2: i64,
    pub y2: i64,
    pub prev: i64,
    pub next: i64,
}

impl Foothold {
    pub fn load(root: nx::Node) -> anyhow::Result<Vec<Self>> {
        let mut footholds = Vec::new();

        for category in root.iter() {
            for data in category.iter() {
                let x1 = data.get("x1").integer().unwrap_or_default();
                let y1 = data.get("y1").integer().unwrap_or_default();
                let x2 = data.get("x2").integer().unwrap_or_default();
                let y2 = data.get("y2").integer().unwrap_or_default();
                let prev = data.get("prev").integer().unwrap_or_default();
                let next = data.get("next").integer().unwrap_or_default();

                let foothold = Foothold {
                    x1,
                    y1,
                    x2,
                    y2,
                    prev,
                    next,
                };

                footholds.push(foothold);
            }
        }

        // TODO we may want to put footholds in a better data structure for searching...

        Ok(footholds)
    }
}

/// Converts the given map id to the nx .img node name
fn get_map_img_name(map_id: i32) -> String {
    let mut map_id_str = String::new();

    // Left-pad map id with zeros
    for _ in 0..9 - map_id.to_string().len() {
        map_id_str.push('0');
    }

    map_id_str.push_str(&map_id.to_string());

    let map_name = format!("{}.img", map_id_str);
    map_name
}

///
fn load_npc_string(npc_id: i32, key: &str) -> String {
    let root = DATA.get("String").unwrap().root();
    let npc_root = root.get("Npc.img").unwrap();

    let npc = npc_root.get(&npc_id.to_string());

    if npc.is_none() {
        return String::new();
    }

    let val = npc.unwrap().get(key);

    if val.is_none() {
        return String::new();
    }

    val.unwrap().string().unwrap_or("").to_string()
}
