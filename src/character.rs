#[derive(Debug)]
pub struct Character {
    pub id: i32,
    pub account_id: i32,
    pub world_id: i32,
    pub name: String,
    pub stats: Stats,
    pub job: i32,
    pub style: Style,
    pub map: i32,
    pub spawn_point: i32,
    pub gm: u8,
    pub rank: Rank,
}

impl Character {
    pub fn new(account_id: i32, world_id: i32, name: String, style: Style) -> Self {
        // TODO pass in job, call default function for each job

        Character {
            id: 0, // FIXME
            account_id,
            world_id,
            name,
            stats: Default::default(),
            job: 0, // TODO different for explorer, cygnus, aran, etc.
            style,
            map: 10000,     // FIXME default for beginnner (mushroom town)
            spawn_point: 0, // FIXME
            gm: 0,
            rank: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct Stats {
    pub exp: i32,
    pub gacha_exp: i32,
    pub level: i32,
    pub str: i32,
    pub dex: i32,
    pub luk: i32,
    pub int: i32,
    pub hp: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub mesos: i32,
    pub fame: i32,
    pub ap: i32,
    pub sp: String,
}

impl Default for Stats {
    fn default() -> Stats {
        Stats {
            exp: 0,
            gacha_exp: 0,
            level: 1,
            str: 12,
            dex: 5,
            luk: 4,
            int: 4,
            hp: 50,
            mp: 5,
            max_hp: 50,
            max_mp: 50,
            mesos: 0,
            fame: 0,
            ap: 0,
            sp: "0,0,0,0,0,0,0,0,0,0".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Style {
    pub skin_colour: i32,
    pub gender: u8,
    pub hair: i32,
    pub face: i32,
}

impl Style {
    pub fn new(skin_colour: i32, gender: u8, hair: i32, face: i32) -> Self {
        Style {
            skin_colour,
            gender,
            hair,
            face,
        }
    }
}

#[derive(Default, Debug)]
pub struct Rank {
    pub rank: i32,
    pub rank_move: i32,
    pub job_rank: i32,
    pub job_rank_move: i32,
}
