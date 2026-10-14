//! Verification of Google Sign-In ID tokens.
//!
//! This uses Google's `tokeninfo` endpoint rather than verifying the JWT
//! signature locally. That endpoint does the signature/issuer/expiry checks
//! for us; we only need to confirm it was issued for one of our own OAuth
//! client IDs and that the email on it is verified. This is the simplest
//! correct approach for a small, self-hosted instance, matching the existing
//! hCaptcha verification pattern in `util::captcha`.

use revolt_config::config;
use revolt_result::{create_error, Result};
use std::sync::LazyLock;

use reqwest::Client;

static CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[derive(Deserialize)]
struct GoogleTokenInfo {
    aud: String,
    sub: String,
    email: Option<String>,
    #[serde(default)]
    email_verified: Option<String>,
}

/// Information extracted from a verified Google ID token.
pub struct GoogleIdentity {
    /// Google's stable, unique identifier for this user ("sub" claim)
    pub google_user_id: String,
    /// The verified email address associated with this Google account
    pub email: String,
}

/// Verify a Google Sign-In ID token and extract the user's identity from it.
///
/// Returns `GoogleAuthNotConfigured` if no client ID has been configured
/// for this instance, and `GoogleAuthFailed` for anything else that doesn't
/// check out (bad/expired token, wrong audience, unverified email).
pub async fn verify_google_id_token(id_token: &str) -> Result<GoogleIdentity> {
    let config = config().await;
    let oauth = &config.google_oauth;

    if oauth.client_id.is_empty() && oauth.android_client_id.is_empty() {
        return Err(create_error!(GoogleAuthNotConfigured));
    }

    let info = CLIENT
        .get("https://oauth2.googleapis.com/tokeninfo")
        .query(&[("id_token", id_token)])
        .send()
        .await
        .map_err(|_| create_error!(GoogleAuthFailed))?
        .json::<GoogleTokenInfo>()
        .await
        .map_err(|_| create_error!(GoogleAuthFailed))?;

    let aud_ok = (!oauth.client_id.is_empty() && info.aud == oauth.client_id)
        || (!oauth.android_client_id.is_empty() && info.aud == oauth.android_client_id);

    if !aud_ok {
        return Err(create_error!(GoogleAuthFailed));
    }

    if info.email_verified.as_deref() != Some("true") {
        return Err(create_error!(GoogleAuthFailed));
    }

    let email = info.email.ok_or_else(|| create_error!(GoogleAuthFailed))?;

    Ok(GoogleIdentity {
        google_user_id: info.sub,
        email,
    })
}
