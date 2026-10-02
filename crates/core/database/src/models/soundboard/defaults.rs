use std::sync::LazyLock;

use reqwest::Client;
use reqwest::multipart::{Form, Part};
use revolt_config::Settings;
use revolt_result::{create_error, Result};
use ulid::Ulid;

use crate::{Database, File, SoundboardSound, SoundboardSoundParent};

/// A default soundboard sound shipped with this fork, embedded at compile
/// time so the binary is self-contained (no network fetch at runtime).
pub struct DefaultSound {
    pub name: &'static str,
    pub bytes: &'static [u8],
}

/// Sounds seeded onto every new personal soundboard and every new server's
/// soundboard. These are original, royalty-free assets committed to this
/// repository at `crates/core/database/assets/default_sounds/`.
pub static DEFAULT_SOUNDS: &[DefaultSound] = &[
    DefaultSound {
        name: "Ding",
        bytes: include_bytes!("../../../assets/default_sounds/ding.wav"),
    },
    DefaultSound {
        name: "Pop",
        bytes: include_bytes!("../../../assets/default_sounds/pop.wav"),
    },
    DefaultSound {
        name: "Chime",
        bytes: include_bytes!("../../../assets/default_sounds/chime.wav"),
    },
    DefaultSound {
        name: "Buzz",
        bytes: include_bytes!("../../../assets/default_sounds/buzz.wav"),
    },
    DefaultSound {
        name: "Boop",
        bytes: include_bytes!("../../../assets/default_sounds/boop.wav"),
    },
    DefaultSound {
        name: "Whoosh",
        bytes: include_bytes!("../../../assets/default_sounds/whoosh.wav"),
    },
];

/// Shared HTTP client for talking to Autumn, mirroring the pattern already
/// used for outbound HTTP calls elsewhere in this crate (see
/// `util::captcha::check_captcha`).
static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

/// Upload one of our default sounds directly to Autumn, server-side, and
/// return the resulting Autumn file id.
///
/// NOTE on the request shape: Autumn's public upload contract (see
/// `docs/docs/developers/api/uploading-files.md` in this repo, and the
/// vendored service at `crates/services/autumn/src/api.rs`) is:
///   POST {autumn_base_url}/{tag}
///   multipart/form-data, single field named "file"
/// This matches what is implemented below (field name "file", tag
/// "attachments", confirmed against the docs/vendored source rather than
/// guessed).
///
/// CAVEAT (flagged in the final report, not just here): Autumn's
/// `upload_file` handler requires a session or bot authentication token
/// (`security(session_token, bot_token)` in its OpenAPI annotation) tied to
/// a real `User`. This helper has no such user-bound token to present, so
/// as written this call will be rejected by an unmodified Autumn with 401
/// Unauthorized until Autumn (or this fork) adds a service-to-service
/// upload exception. Because every caller of this function treats upload
/// failure as best-effort (see `seed_default_soundboard_sounds`), that
/// means default sounds silently fail to seed today rather than breaking
/// account/server creation — but it does mean the feature does not yet
/// work end-to-end without that follow-up change.
async fn upload_default_sound_to_autumn(
    settings: &Settings,
    bytes: &'static [u8],
    filename: &str,
) -> Result<String> {
    let part = Part::bytes(bytes)
        .file_name(filename.to_owned())
        .mime_str("audio/wav")
        .map_err(|_| create_error!(InternalError))?;

    let form = Form::new().part("file", part);

    let response = CLIENT
        .post(format!("{}/attachments", settings.hosts.autumn))
        .multipart(form)
        .send()
        .await
        .map_err(|_| create_error!(InternalError))?;

    if !response.status().is_success() {
        return Err(create_error!(InternalError));
    }

    #[derive(serde::Deserialize)]
    struct UploadResponse {
        id: String,
    }

    response
        .json::<UploadResponse>()
        .await
        .map(|body| body.id)
        .map_err(|_| create_error!(InternalError))
}

/// Seed the default soundboard sounds for a newly created user (`parent` =
/// `Detached`) or a newly created server (`parent` = `Server { id }`).
///
/// This is a best-effort nicety: any failure (Autumn unreachable, Autumn
/// rejecting the upload, or a database error) is logged and the function
/// simply moves on, since nothing here is allowed to fail the account or
/// server creation request that triggered it.
pub async fn seed_default_soundboard_sounds(
    db: &Database,
    settings: &Settings,
    parent: SoundboardSoundParent,
    creator_id: &str,
) {
    for default in DEFAULT_SOUNDS {
        let filename = format!("{}.wav", default.name.to_lowercase());

        let file_id = match upload_default_sound_to_autumn(settings, default.bytes, &filename).await {
            Ok(id) => id,
            Err(err) => {
                warn!("Failed to upload default soundboard sound {:?} to Autumn: {err:?}", default.name);
                continue;
            }
        };

        let sound_id = Ulid::new().to_string();

        let attachment =
            match File::use_soundboard_sound(db, &file_id, &sound_id, creator_id).await {
                Ok(attachment) => attachment,
                Err(err) => {
                    warn!("Failed to claim default soundboard sound {:?} ({file_id}) in Autumn: {err:?}", default.name);
                    continue;
                }
            };

        let sound = SoundboardSound {
            id: sound_id,
            parent: parent.clone(),
            creator_id: creator_id.to_owned(),
            name: default.name.to_owned(),
            emoji: None,
            file_id: attachment.id,
            volume: 100,
        };

        if let Err(err) = sound.create(db).await {
            warn!("Failed to create default soundboard sound {:?}: {err:?}", default.name);
        }
    }
}
