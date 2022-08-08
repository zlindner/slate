use sqlx::FromRow;

pub(crate) mod pet;
pub use self::pet::Pet;

#[derive(FromRow)]
pub struct Character {
    pub id: i32,
    pub account_id: i32,
    pub world_id: i32,
    pub name: String,
    pub level: i16,
    pub exp: i32,
    pub gacha_exp: i32,
    pub str: i16,
    pub dex: i16,
    pub luk: i16,
    pub int: i16,
    pub hp: i16,
    pub mp: i16,
    pub max_hp: i16,
    pub max_mp: i16,
    pub mesos: i32,
    pub job: i16,
    pub skin_colour: i16,
    pub gender: i16,
    pub fame: i16,
    pub hair: i32,
    pub face: i32,
    pub ap: i16,
    pub sp: String,
    pub map: i32,
    pub spawn_point: i16,
    pub gm: i16,
    pub rank: i32,
    pub rank_move: i32,
    pub job_rank: i32,
    pub job_rank_move: i32,

    #[sqlx(default)]
    pub pets: Vec<Pet>,
}
