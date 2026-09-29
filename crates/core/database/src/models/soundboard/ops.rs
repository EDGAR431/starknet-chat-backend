use revolt_result::Result;

use crate::{PartialSoundboardSound, SoundboardSound};

#[cfg(feature = "mongodb")]
mod mongodb;
mod reference;

#[async_trait]
pub trait AbstractSoundboard: Sync + Send {
    /// Insert soundboard sound into database.
    async fn insert_soundboard_sound(&self, sound: &SoundboardSound) -> Result<()>;

    /// Fetch a soundboard sound by its id
    async fn fetch_soundboard_sound(&self, id: &str) -> Result<SoundboardSound>;

    /// Fetch soundboard sounds by their parent id
    async fn fetch_soundboard_sounds_by_parent_id(
        &self,
        parent_id: &str,
    ) -> Result<Vec<SoundboardSound>>;

    /// Update soundboard sound with new information
    async fn update_soundboard_sound(
        &self,
        sound_id: &str,
        partial: &PartialSoundboardSound,
    ) -> Result<()>;

    /// Detach a soundboard sound by its id
    async fn detach_soundboard_sound(&self, sound: &SoundboardSound) -> Result<()>;
}
