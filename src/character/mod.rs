use sqlx::{postgres::PgRow, Row};

use self::pet::Pet;

mod pet;

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

    pub pets: Vec<Pet>,
}

#[derive(Debug)]
pub struct Stats {
    pub exp: i32,
    pub gacha_exp: i32,
    pub level: i16,
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

#[derive(Debug)]
pub struct Style {
    pub skin_colour: i32,
    pub gender: u8,
    pub hair: i32,
    pub face: i32,
}

#[derive(Default, Debug)]
pub struct Rank {
    pub rank: i32,
    pub rank_move: i32,
    pub job_rank: i32,
    pub job_rank_move: i32,
}

impl Character {
    pub fn new(account_id: i32, world_id: i32, name: String, style: Style) -> Self {
        // TODO pass in job, call default function for each job

        Self {
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
            pets: Vec::new(),
        }
    }

    // TODO should probably change this to load() in the future, character data spans
    // across several tables/queries
    pub fn from_row(row: &PgRow) -> Self {
        Self {
            id: row.get::<i32, _>("id"),
            account_id: row.get::<i32, _>("account_id"),
            world_id: row.get::<i32, _>("world_id"),
            name: row.get::<String, _>("name"),
            stats: Stats::from_row(row),
            job: row.get::<i32, _>("job"),
            style: Style::from_row(row),
            map: row.get::<i32, _>("map"),
            spawn_point: row.get::<i32, _>("spawn_point"),
            gm: row.get::<i16, _>("gm") as u8,
            rank: Rank::from_row(row),
            // TODO load from pets table
            pets: Vec::new(),
        }
    }

    // TODO new_explorer, new_cygnus, new_aran, etc.
}

impl Stats {
    fn from_row(row: &PgRow) -> Self {
        Stats {
            exp: row.get::<i32, _>("exp"),
            gacha_exp: row.get::<i32, _>("gacha_exp"),
            level: row.get::<i16, _>("level"),
            str: row.get::<i32, _>("str"),
            dex: row.get::<i32, _>("dex"),
            luk: row.get::<i32, _>("luk"),
            int: row.get::<i32, _>("int"),
            hp: row.get::<i32, _>("hp"),
            mp: row.get::<i32, _>("mp"),
            max_hp: row.get::<i32, _>("max_hp"),
            max_mp: row.get::<i32, _>("max_mp"),
            mesos: row.get::<i32, _>("mesos"),
            fame: row.get::<i32, _>("fame"),
            ap: row.get::<i32, _>("ap"),
            sp: row.get::<String, _>("sp"),
        }
    }
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

impl Style {
    fn from_row(row: &PgRow) -> Self {
        Self {
            skin_colour: row.get::<i32, _>("skin_colour"),
            gender: row.get::<i16, _>("gender") as u8,
            hair: row.get::<i32, _>("hair"),
            face: row.get::<i32, _>("face"),
        }
    }
}

impl Rank {
    fn from_row(row: &PgRow) -> Self {
        Self {
            rank: row.get::<i32, _>("rank"),
            rank_move: row.get::<i32, _>("rank_move"),
            job_rank: row.get::<i32, _>("job_rank"),
            job_rank_move: row.get::<i32, _>("job_rank_move"),
        }
    }
}
