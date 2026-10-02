#[cfg(feature = "validator")]
use validator::Validate;

auto_derived!(
    /// A sound on a soundboard
    pub struct SoundboardSound {
        /// Unique Id
        #[cfg_attr(feature = "serde", serde(rename = "_id"))]
        pub id: String,
        /// What owns this sound
        pub parent: SoundboardSoundParent,
        /// Uploader user id
        pub creator_id: String,
        /// Sound name
        pub name: String,
        /// Emoji representing this sound
        #[cfg_attr(
            feature = "serde",
            serde(skip_serializing_if = "Option::is_none")
        )]
        pub emoji: Option<String>,
        /// Autumn file id of the audio clip
        pub file_id: String,
        /// Playback volume as a percentage, from 0 to 200 (100 is unchanged)
        pub volume: u16,
        /// Whether this is a permanent, protected default sound that cannot be deleted
        #[cfg_attr(feature = "serde", serde(default))]
        pub is_default: bool,
    }

    /// Parent Id of the soundboard sound
    #[serde(tag = "type")]
    pub enum SoundboardSoundParent {
        Server { id: String },
        Detached,
    }

    /// Create a new soundboard sound
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataCreateSoundboardSound {
        /// Sound name
        #[validate(length(min = 1, max = 32))]
        pub name: String,
        /// Parent information
        pub parent: SoundboardSoundParent,
        /// Emoji representing this sound
        #[serde(default)]
        pub emoji: Option<String>,
        /// Autumn file id of the already-uploaded audio clip
        pub file_id: String,
        /// Playback volume as a percentage, from 0 to 200 (defaults to 100)
        #[serde(default)]
        pub volume: Option<u16>,
    }

    /// Partial soundboard sound representation
    #[derive(Default)]
    pub struct PartialSoundboardSound {
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub name: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub emoji: Option<String>,
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        pub volume: Option<u16>,
    }

    /// Edit soundboard sound information
    #[cfg_attr(feature = "validator", derive(Validate))]
    pub struct DataEditSoundboardSound {
        /// Sound name
        #[cfg_attr(feature = "validator", validate(length(min = 1, max = 32)))]
        pub name: Option<String>,
        /// Emoji representing this sound
        pub emoji: Option<String>,
        /// Playback volume as a percentage, from 0 to 200
        pub volume: Option<u16>,
    }
);
