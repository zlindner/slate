use crate::Db;
use sqlx::FromRow;

#[derive(FromRow)]
pub struct Keymap {
    pub id: i32,
    pub character_id: i32,
    pub key_id: i32,
    pub key_type: i32,
    pub action: i32,
}

impl Keymap {
    /// Loads the keymaps for a character
    pub async fn load(character_id: i32, db: &Db) -> anyhow::Result<Vec<Self>> {
        let keymaps = sqlx::query_as::<_, Keymap>("SELECT * FROM keymaps WHERE character_id = ?")
            .bind(character_id)
            .fetch_all(db)
            .await?;

        Ok(keymaps)
    }
}
