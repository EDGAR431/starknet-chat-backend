use revolt_result::Result;

use crate::ReferenceDb;
use crate::SoundboardSoundParent;
use crate::{PartialSoundboardSound, SoundboardSound};

use super::AbstractSoundboard;

#[async_trait]
impl AbstractSoundboard for ReferenceDb {
    /// Insert soundboard sound into database.
    async fn insert_soundboard_sound(&self, sound: &SoundboardSound) -> Result<()> {
        let mut sounds = self.soundboard_sounds.lock().await;
        if sounds.contains_key(&sound.id) {
            Err(create_database_error!("insert", "soundboard_sound"))
        } else {
            sounds.insert(sound.id.to_string(), sound.clone());
            Ok(())
        }
    }

    /// Fetch a soundboard sound by its id
    async fn fetch_soundboard_sound(&self, id: &str) -> Result<SoundboardSound> {
        let sounds = self.soundboard_sounds.lock().await;
        sounds
            .get(id)
            .cloned()
            .ok_or_else(|| create_error!(NotFound))
    }

    /// Fetch soundboard sounds by their parent id
    async fn fetch_soundboard_sounds_by_parent_id(
        &self,
        parent_id: &str,
    ) -> Result<Vec<SoundboardSound>> {
        let sounds = self.soundboard_sounds.lock().await;
        Ok(sounds
            .values()
            .filter(|sound| match &sound.parent {
                SoundboardSoundParent::Server { id } => id == parent_id,
                _ => false,
            })
            .cloned()
            .collect())
    }

    /// Update soundboard sound with new information
    async fn update_soundboard_sound(
        &self,
        sound_id: &str,
        partial: &PartialSoundboardSound,
    ) -> Result<()> {
        let mut sounds = self.soundboard_sounds.lock().await;
        if let Some(sound) = sounds.get_mut(sound_id) {
            if let Some(name) = partial.name.clone() {
                sound.name = name;
            }
            if partial.emoji.is_some() {
                sound.emoji = partial.emoji.clone();
            }
            if let Some(volume) = partial.volume {
                sound.volume = volume;
            }
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }

    /// Detach a soundboard sound by its id
    async fn detach_soundboard_sound(&self, sound: &SoundboardSound) -> Result<()> {
        let mut sounds = self.soundboard_sounds.lock().await;
        if let Some(existing) = sounds.get_mut(&sound.id) {
            existing.parent = SoundboardSoundParent::Detached;
            Ok(())
        } else {
            Err(create_error!(NotFound))
        }
    }
}
