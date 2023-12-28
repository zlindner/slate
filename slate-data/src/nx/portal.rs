use nx::GenericNode;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Portal {
    pub id: i32,
    pub name: String,
    pub script: String,
    pub target: String,
    pub target_map_id: i32,
    pub type_: i32,
    pub x: i32,
    pub y: i32,
}

impl Portal {
    pub fn load(root: nx::Node) -> anyhow::Result<HashMap<i32, Self>> {
        let mut portals = HashMap::new();
        let mut next_door_portal_id = 0x80;

        for data in root.iter() {
            let name = data.get("pn").string().unwrap_or_default().to_string();
            let script = data.get("script").string().unwrap_or_default().to_string();
            let target = data.get("tn").string().unwrap_or_default().to_string();
            let target_map_id = data.get("tm").integer().unwrap_or_default() as i32;
            let type_ = data.get("pt").integer().unwrap_or_default() as i32;
            let x = data.get("x").integer().unwrap_or_default() as i32;
            let y = data.get("x").integer().unwrap_or_default() as i32;

            let id = match type_ {
                // Door portal
                6 => {
                    let id = next_door_portal_id;
                    next_door_portal_id += 1;
                    id
                }
                _ => data.name().parse().unwrap(),
            };

            let portal = Portal {
                id,
                name,
                script,
                target,
                target_map_id,
                type_,
                x,
                y,
            };

            log::debug!("Loaded portal: {:?}", portal);
            portals.insert(id, portal);
        }

        Ok(portals)
    }
}
