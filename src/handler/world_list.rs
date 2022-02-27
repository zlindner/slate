use crate::{client::Client, Result};

#[derive(Debug)]
pub struct WorldList;

impl WorldList {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(self, client: &mut Client) -> Result<()> {
        let shared = &client.shared;

        for world in shared.worlds.iter() {
            log::debug!("world: {:?}", world);
        }

        Ok(())
    }
}
