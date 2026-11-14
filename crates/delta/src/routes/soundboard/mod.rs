use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod sound_create;
mod sound_delete;
mod sound_fetch_all;
mod sound_fetch_personal;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        sound_create::create_sound,
        sound_delete::delete_sound,
        sound_fetch_all::fetch_server_sounds,
        sound_fetch_personal::fetch_personal_sounds
    ]
}
