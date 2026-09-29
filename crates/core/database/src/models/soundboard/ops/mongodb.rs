use bson::Document;
use revolt_result::Result;

use crate::MongoDb;
use crate::{PartialSoundboardSound, SoundboardSound};

use super::AbstractSoundboard;

static COL: &str = "soundboard_sounds";

#[async_trait]
impl AbstractSoundboard for MongoDb {
    /// Insert soundboard sound into database.
    async fn insert_soundboard_sound(&self, sound: &SoundboardSound) -> Result<()> {
        query!(self, insert_one, COL, &sound).map(|_| ())
    }

    /// Fetch a soundboard sound by its id
    async fn fetch_soundboard_sound(&self, id: &str) -> Result<SoundboardSound> {
        query!(self, find_one_by_id, COL, id)?.ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch soundboard sounds by their parent id
    async fn fetch_soundboard_sounds_by_parent_id(
        &self,
        parent_id: &str,
    ) -> Result<Vec<SoundboardSound>> {
        query!(
            self,
            find,
            COL,
            doc! {
                "parent.id": parent_id
            }
        )
    }

    /// Update soundboard sound with new information
    async fn update_soundboard_sound(
        &self,
        sound_id: &str,
        partial: &PartialSoundboardSound,
    ) -> Result<()> {
        query!(self, update_one_by_id, COL, sound_id, partial, vec![], None).map(|_| ())
    }

    /// Detach a soundboard sound by its id
    async fn detach_soundboard_sound(&self, sound: &SoundboardSound) -> Result<()> {
        self.col::<Document>(COL)
            .update_one(
                doc! {
                    "_id": &sound.id
                },
                doc! {
                    "$set": {
                        "parent": {
                            "type": "Detached"
                        }
                    }
                },
            )
            .await
            .map(|_| ())
            .map_err(|_| create_database_error!("update_one", COL))
    }
}
