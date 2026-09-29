use revolt_database::{
    util::{permissions::DatabasePermissionQuery, reference::Reference},
    Database, SoundboardSoundParent, User,
};
use revolt_permissions::{calculate_server_permissions, ChannelPermission};
use revolt_result::Result;

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
            SoundboardSoundParent::Detached => return Ok(EmptyResponse),
        };
    }

    sound.delete(db).await?;

    Ok(EmptyResponse)
}
