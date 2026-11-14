use revolt_database::{
    util::reference::Reference,
    Database, SoundboardSoundParent, User,
};
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

    match &sound.parent {
        // Server sounds can only be removed by the server owner, so nobody
        // (not even their uploader or an admin) can delete them on a whim.
        SoundboardSoundParent::Server { id } => {
            let server = db.fetch_server(id.as_str()).await?;
            if server.owner != user.id {
                return Err(create_error!(MissingPermission {
                    permission: "ServerOwner".to_string()
                }));
            }
        }
        // Personal sounds can only be deleted by the user who created them.
        SoundboardSoundParent::Detached => {
            if sound.creator_id != user.id {
                return Err(create_error!(NotFound));
            }
        }
    }

    sound.delete(db).await?;

    Ok(EmptyResponse)
}
