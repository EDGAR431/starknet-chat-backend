use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_result::Result;

use rocket::{serde::json::Json, State};

/// # Fetch Personal Soundboard
///
/// Fetch every sound on the current user's personal soundboard.
#[openapi(tag = "Soundboard")]
#[get("/personal")]
pub async fn fetch_personal_sounds(
    db: &State<Database>,
    user: User,
) -> Result<Json<Vec<v0::SoundboardSound>>> {
    db.fetch_personal_soundboard_sounds(&user.id)
        .await
        .map(|v| v.into_iter().map(Into::into).collect())
        .map(Json)
}
