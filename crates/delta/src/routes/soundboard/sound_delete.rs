use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, SoundboardSoundParent, User,
};
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::{create_error, Result};

use rocket::State;
use rocket_empty::EmptyResponse;

/// # Delete Soundboard Sound
///
/// Delete a sound from a soundboard by its id.
#[openapi(tag = "Soundboard")]
#[delete("/sound/<sound_id>")]
pub async fn delete_sound(
    db: &State<Database>,
    user: User,
    sound_id: Reference<'_>,
) -> Result<EmptyResponse> {
    let sound = db.fetch_soundboard_sound(sound_id.id).await?;

    // Default (starter-pack) sounds are permanently protected and can never
    // be deleted by anyone, including a server admin with
    // ManageCustomisation or the sound's own creator.
    if sound.is_default {
        return Err(create_error!(InvalidOperation));
    }

    // If we uploaded the sound, we may delete it ourselves
    if sound.creator_id != user.id {
        match &sound.parent {
            SoundboardSoundParent::Server { id } => {
                let server = db.fetch_server(id.as_str()).await?;

                let mut query = DatabasePermissionQuery::new(db, &user).server(&server);
                calculate_server_permissions(&mut query)
                    .await
                    .throw_if_lacking_channel_permission(ChannelPermission::ManageCustomisation)?;
            }
            // Detached (personal) sounds have no server-level permission
            // concept to fall back on, so a non-creator never has
            // permission to delete them.
            SoundboardSoundParent::Detached => return Err(create_error!(NotFound)),
        };
    }

    sound.delete(db).await?;

    Ok(EmptyResponse)
}
