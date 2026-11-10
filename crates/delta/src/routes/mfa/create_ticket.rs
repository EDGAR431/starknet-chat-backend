//! Create a new MFA ticket or validate an existing one.
//! PUT /mfa/ticket
use revolt_result::{Result, create_error};
use revolt_database::{Account, Database, MFATicket, UnvalidatedTicket};
use revolt_models::v0;
use rocket::serde::json::Json;
use rocket::State;


/// # Create MFA ticket
///
/// Create a new MFA ticket or validate an existing one.
#[openapi(tag = "MFA")]
#[put("/ticket", data = "<data>")]
pub async fn create_ticket(
    db: &State<Database>,
    account: Option<Account>,
    existing_ticket: Option<UnvalidatedTicket>,
    data: Json<v0::MFAResponse>,
) -> Result<Json<v0::MFATicket>> {
    // Find the relevant account
    let mut account = match (account, existing_ticket) {
        (Some(_), Some(_)) => return Err(create_error!(OperationFailed)),
        (Some(account), _) => account,
        (_, Some(ticket)) => {
            db.delete_ticket(&ticket.id).await?;
            db.fetch_account(&ticket.account_id).await?
        }
        _ => return Err(create_error!(InvalidToken)),
    };

    // Validate the MFA response
    account
        .consume_mfa_response(db, data.into_inner(), None)
        .await?;

    // Create a new ticket for this account
    let ticket = MFATicket::new(account.id, true);
    ticket.save(db).await?;
    Ok(Json(ticket.into()))
}

#[cfg(test)]
mod tests {
    use crate::{rocket, util::test::TestHarness};
    use revolt_database::Totp;
    use rocket::http::{Header, Status};
    use revolt_models::v0;
    use revolt_result::{Error, ErrorType};

    #[rocket::async_test]
    async fn success() {
        let harness = TestHarness::new().await;
        let (_, session, _) = harness.new_user().await;

        let res = harness.client
            .put("/auth/mfa/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "password": "password_insecure"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(res.into_json::<v0::MFATicket>().await.unwrap().validated);
    }

    #[rocket::async_test]
    async fn success_totp() {
        let harness = TestHarness::new().await;
        let (mut account, session, _) = harness.new_user().await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "secret".to_string(),
        };
        account.save(&harness.db).await.unwrap();

        let res = harness.client
            .put("/auth/mfa/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "totp_code": Totp::Enabled {
                        secret: "secret".to_string(),
                    }.generate_code().unwrap()
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Ok);
        assert!(res.into_json::<v0::MFATicket>().await.is_some());
    }

    #[rocket::async_test]
    async fn failure_totp() {
        let harness = TestHarness::new().await;
        let (mut account, session, _) = harness.new_user().await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "secret".to_string(),
        };
        account.save(&harness.db).await.unwrap();

        let res = harness.client
            .put("/auth/mfa/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "totp_code": "000000"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
        assert!(matches!(
            res.into_json::<Error>().await.unwrap().error_type,
            ErrorType::InvalidToken,
        ));
    }

    #[rocket::async_test]
    async fn failure_no_totp() {
        let harness = TestHarness::new().await;
        let (mut account, session, _) = harness.new_user().await;

        account.mfa.totp_token = Totp::Enabled {
            secret: "secret".to_string(),
        };
        account.save(&harness.db).await.unwrap();

        let res = harness.client
            .put("/auth/mfa/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(
                json!({
                    "password": "this is the wrong mfa method"
                })
                .to_string(),
            )
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert!(matches!(
            res.into_json::<Error>().await.unwrap().error_type,
            ErrorType::DisallowedMFAMethod,
        ));
    }
}

#[cfg(test)]
mod google_repro {
    use crate::{rocket, util::test::TestHarness};
    use iso8601_timestamp::{Duration, Timestamp};
    use revolt_database::{Account, PasswordReset};
    use rocket::http::{Header, Status};

    #[rocket::async_test]
    async fn google_account_cannot_get_mfa_ticket_with_password() {
        // TestHarness::new() also needs RabbitMQ; build only what we need.
        struct H {
            client: rocket::local::asynchronous::Client,
            db: revolt_database::Database,
        }
        let client = rocket::local::asynchronous::Client::tracked(crate::web().await)
            .await
            .expect("rocket");
        let db = client.rocket().state::<revolt_database::Database>().unwrap().clone();
        let harness = H { client, db };

        let mut account = Account::new_from_google(
            &harness.db,
            format!("{}@gmail.com", TestHarness::rand_string()),
            TestHarness::rand_string(),
        )
        .await
        .expect("google account");

        let session = account
            .create_session(&harness.db, String::new())
            .await
            .expect("session");

        eprintln!("MFA methods offered: {:?}", account.mfa.get_methods());

        // 1. The UI's only option: type a password. Try a few guesses.
        for guess in ["password_insecure", "", "Contraseña123!"] {
            let res = harness
                .client
                .put("/auth/mfa/ticket")
                .header(Header::new("X-Session-Token", session.token.clone()))
                .body(serde_json::json!({ "password": guess }).to_string())
                .dispatch()
                .await;
            eprintln!("ticket with password {:?} -> {}", guess, res.status());
            assert_ne!(res.status(), Status::Ok);
        }

        // 2. Escape hatch: password reset flow (email step simulated).
        account.password_reset = Some(PasswordReset {
            token: "repro-token".to_string(),
            expiry: Timestamp::now_utc() + Duration::seconds(300),
        });
        account.save(&harness.db).await.unwrap();

        let new_pw = "correct horse battery staple 8841";
        let res = harness
            .client
            .patch("/auth/account/reset_password")
            .header(rocket::http::ContentType::JSON)
            .body(
                serde_json::json!({
                    "token": "repro-token",
                    "password": new_pw
                })
                .to_string(),
            )
            .dispatch()
            .await;
        eprintln!("reset_password -> {}", res.status());

        // 3. After reset, does the same dialog work?
        let res = harness
            .client
            .put("/auth/mfa/ticket")
            .header(Header::new("X-Session-Token", session.token.clone()))
            .body(serde_json::json!({ "password": new_pw }).to_string())
            .dispatch()
            .await;
        eprintln!("ticket after reset -> {}", res.status());
    }
}
