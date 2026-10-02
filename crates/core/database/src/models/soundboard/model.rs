use revolt_models::v0;
use revolt_result::Result;

use crate::events::client::EventV1;
use crate::Database;

fn default_volume() -> u16 {
    100
}

auto_derived_partial!(
    /// Soundboard sound
    pub struct SoundboardSound {
        /// Unique Id
        #[serde(rename = "_id")]
        pub id: String,
        /// What owns this sound
        pub parent: SoundboardSoundParent,
        /// Uploader user id
        pub creator_id: String,
        /// Sound name
        pub name: String,
        /// Emoji representing this sound
        #[serde(skip_serializing_if = "Option::is_none")]
        pub emoji: Option<String>,
        /// Autumn file id of the audio clip
        pub file_id: String,
        /// Playback volume as a percentage, from 0 to 200 (100 is unchanged)
        #[serde(default = "default_volume")]
        pub volume: u16,
        /// Whether this is a permanent, protected default sound that cannot be deleted
        ///
        /// This flag is internal-only: it is never part of `PartialSoundboardSound`
        /// and can never be set via any edit route, only at seed time (see
        /// `defaults.rs`).
        #[serde(default)]
        pub is_default: bool,
    },
    "PartialSoundboardSound"
);

auto_derived!(
    /// Parent Id of the soundboard sound
    #[serde(tag = "type")]
    pub enum SoundboardSoundParent {
        Server { id: String },
        Detached,
    }
);

#[allow(clippy::disallowed_methods)]
impl SoundboardSound {
    /// Get parent id
    fn parent(&self) -> &str {
        match &self.parent {
            SoundboardSoundParent::Server { id } => id,
            SoundboardSoundParent::Detached => "",
        }
    }

    /// Create a soundboard sound
    pub async fn create(&self, db: &Database) -> Result<()> {
        db.insert_soundboard_sound(self).await?;

        EventV1::SoundboardSoundCreate(self.clone().into())
            .p(self.parent().to_string())
            .await;

        Ok(())
    }

    /// Delete a soundboard sound
    pub async fn delete(&self, db: &Database) -> Result<()> {
        EventV1::SoundboardSoundDelete {
            id: self.id.to_string(),
        }
        .p(self.parent().to_string())
        .await;

        db.detach_soundboard_sound(self).await
    }

    /// Update a soundboard sound
    pub async fn update(&mut self, db: &Database, partial: PartialSoundboardSound) -> Result<()> {
        if let Some(name) = partial.name.clone() {
            self.name = name;
        }

        if partial.emoji.is_some() {
            self.emoji = partial.emoji.clone();
        }

        if let Some(volume) = partial.volume {
            self.volume = volume;
        }

        db.update_soundboard_sound(&self.id, &partial).await?;

        EventV1::SoundboardSoundUpdate {
            id: self.id.clone(),
            data: v0::PartialSoundboardSound {
                name: partial.name.clone(),
                emoji: partial.emoji.clone(),
                volume: partial.volume,
            },
        }
        .p(self.parent().to_string())
        .await;

        Ok(())
    }

    /// Generates a PartialSoundboardSound containing the data which has changed in an update
    pub fn generate_diff(&self, partial: &PartialSoundboardSound) -> PartialSoundboardSound {
        let mut before = PartialSoundboardSound::default();

        generate_diff!(
            self, before, partial, remove,
            (
                name,
                volume,
            )
        );

        if partial.emoji.is_some() {
            before.emoji = self.emoji.clone();
        }

        before
    }
}
