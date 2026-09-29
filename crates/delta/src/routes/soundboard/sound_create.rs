use revolt_database::{Database, File, SoundboardSound, User, util::permissions::DatabasePermissionQuery};
use revolt_models::v0;
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};
use ulid::Ulid;
use validator::Validate;

use rocket::{serde::json::Json, State};

/// Soundboards are capped locally rather than through a new config field, to
/// avoid requiring every self-hosted Config.toml to be edited on upgrade.
const MAX_SOUNDS_PER_SERVER: usize = 100;
const MAX_VOLUME: u16 = 200;

/// # Create New Soundboard Sound
///
/// Add a sound to a server's soundboard by its Autumn upload id.
#[openapi(tag = "Soundboard")]
#[post("/sound", data = "<data>")]
pub async fn create_sound(
    db: &State<Database>,
    user: User,
    data: Json<v0::DataCreateSoundboardSound>,
) -> Result<Json<v0::SoundboardSound>> {
    let data = data.into_inner();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    // Validate we have permission to write into parent
    match &data.parent {
        v0::SoundboardSoundParent::Server { id } => {
            let server = db.fetch_server(id).await?;

            let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
            calculate_server_permissions(&mut query)
                .await
                .throw_if_lacking_channel_permission(ChannelPermission::ManageCustomisation)?;

            let sounds = db.fetch_soundboard_sounds_by_parent_id(&server.id).await?;
            if sounds.len() >= MAX_SOUNDS_PER_SERVER {
                return Err(create_error!(InvalidOperation));
            }
        }
        v0::SoundboardSoundParent::Detached => return Err(create_error!(InvalidOperation)),
    };

    let sound_id = Ulid::new().to_string();

    // Claim the uploaded attachment for this sound
    let attachment =
        File::use_soundboard_sound(db, &data.file_id, &sound_id, &user.id).await?;

    if !attachment.content_type.starts_with("audio/") {
        return Err(create_error!(InvalidOperation));
    }

    let volume = data.volume.unwrap_or(100).min(MAX_VOLUME);

    let sound = SoundboardSound {
        id: sound_id,
        parent: data.parent.clone().into(),
        creator_id: user.id.clone(),
        name: data.name,
        emoji: data.emoji,
        file_id: attachment.id,
        volume,
    };

    sound.create(db).await?;

    Ok(Json(sound.into()))
}
