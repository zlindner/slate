use sqlx::{
    types::{
        chrono::{DateTime, Utc},
        Json,
    },
    Decode, Encode, FromRow,
};

#[derive(FromRow)]
pub struct LoginSessionData {
    pub account_id: i32,
    pub character_id: i32,
    pub world_id: i32,
    pub channel_id: i32,
    pub map_id: i32,
    pub login_attempts: i32,
    pub pin: String,
    pub pin_attempts: i32,
    pub pic: String,
    pub pic_attempts: i32,
}

impl Default for LoginSessionData {
    fn default() -> Self {
        Self {
            account_id: -1,
            character_id: -1,
            world_id: -1,
            channel_id: -1,
            map_id: -1,
            login_attempts: 0,
            pin: String::new(),
            pin_attempts: 0,
            pic: String::new(),
            pic_attempts: 0,
        }
    }
}

#[derive(FromRow)]
pub struct Account {
    pub id: i32,
    pub name: String,
    pub password: String,
    pub pin: String,
    pub pic: String,
    pub state: LoginState,
    pub banned: bool,
    pub accepted_tos: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub gender: i32,
}

#[derive(Decode, Encode)]
pub enum LoginState {
    LoggedIn,
    Transitioning,
    LoggedOut,
}

impl sqlx::Type<sqlx::MySql> for LoginState {
    fn type_info() -> <sqlx::MySql as sqlx::Database>::TypeInfo {
        <str as sqlx::Type<sqlx::MySql>>::type_info()
    }

    fn compatible(ty: &<sqlx::MySql as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<sqlx::MySql>>::compatible(ty)
    }
}

#[derive(FromRow)]
pub struct World {
    pub id: i32,
    pub connected_players: i32,
}

#[derive(FromRow)]
pub struct Character {
    pub id: i32,
    pub account_id: i32,
    pub world_id: i32,
    pub name: String,
    pub level: i32,
    pub exp: i32,
    pub gacha_exp: i32,
    pub str: i32,
    pub dex: i32,
    pub luk: i32,
    pub int: i32,
    pub hp: i32,
    pub mp: i32,
    pub max_hp: i32,
    pub max_mp: i32,
    pub mesos: i32,
    pub job: i32,
    pub skin_colour: i32,
    pub gender: i32,
    pub fame: i32,
    pub hair: i32,
    pub face: i32,
    pub ap: i32,
    pub sp: String,
    pub map: i32,
    pub spawn_point: i32,
    pub gm: i32,
    pub party: Option<i32>,
    pub buddy_capacity: i32,
    pub created_at: DateTime<Utc>,
    pub rank: i32,
    pub rank_move: i32,
    pub job_rank: i32,
    pub job_rank_move: i32,
    pub guild: Option<i32>,
    pub guild_rank: Option<i32>,
    pub equip_slots: i32,
    pub use_slots: i32,
    pub setup_slots: i32,
    pub etc_slots: i32,
    pub cash_slots: i32,
    // items: Vec<Item>,
    // equips: Vec<Equip>,
    // skills: Vec<Skill>,
    // keymaps: Vec<Keymap>,
    // cooldowns: Vec<Cooldown>,
    // quests: Vec<Quest>
}

#[derive(FromRow)]
pub struct Equipment {
    pub id: i32,
    pub item_id: i32,
    pub character_id: i32,
    pub position: i32,
    pub amount: i32,
    pub upgrade_slots: i32,
    pub level: i32,
    pub item_level: i32,
    pub exp: i32,
    pub str: i32,
    pub dex: i32,
    pub int: i32,
    pub luk: i32,
    pub hp: i32,
    pub mp: i32,
    pub w_atk: i32,
    pub m_atk: i32,
    pub w_def: i32,
    pub m_def: i32,
    pub acc: i32,
    pub avoid: i32,
    pub hands: i32,
    pub speed: i32,
    pub jump: i32,
    pub locked: i32,
    pub vicious: i32,
    pub owner: String,
    pub flag: i32,
}
