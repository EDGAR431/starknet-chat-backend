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
mod google_method_tests {
    use crate::{rocket, util::test::TestHarness};
    use revolt_database::{Account, MFAMethod, Totp};
    use rocket::http::{Header, Status};
    use revolt_models::v0;
    use revolt_result::{Error, ErrorType};

    #[rocket::async_test]
    async fn google_account_is_offered_google_method() {
        let harness = TestHarness::new().await;
        let account = Account::new_from_google(
            &harness.db,
            format!("{}@gmail.com", TestHarness::rand_string()),
            TestHarness::rand_string(),
        )
        .await
        .unwrap();
        let session = account.create_session(&harness.db, String::new()).await.unwrap();

        let res = harness
            .client
            .get("/auth/mfa/methods")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;
        assert_eq!(res.status(), Status::Ok);
        let methods = res.into_json::<Vec<v0::MFAMethod>>().await.unwrap();
        assert!(methods.contains(&v0::MFAMethod::Password));
        assert!(methods.contains(&v0::MFAMethod::Google));
    }

    #[rocket::async_test]
    async fn google_not_offered_when_totp_enabled() {
        let harness = TestHarness::new().await;
        let mut account = Account::new_from_google(
            &harness.db,
            format!("{}@gmail.com", TestHarness::rand_string()),
            TestHarness::rand_string(),
        )
        .await
        .unwrap();
        account.mfa.totp_token = Totp::Enabled { secret: "secret".to_string() };
        account.save(&harness.db).await.unwrap();

        assert!(!account.verification_methods().contains(&MFAMethod::Google));
    }

    #[rocket::async_test]
    async fn non_google_account_cannot_use_google_method() {
        let harness = TestHarness::new().await;
        let mut account = Account::new_from_google(
            &harness.db,
            format!("{}@gmail.com", TestHarness::rand_string()),
            TestHarness::rand_string(),
        )
        .await
        .unwrap();
        account.google_id = None;
        account.save(&harness.db).await.unwrap();
        let session = account.create_session(&harness.db, String::new()).await.unwrap();

        let res = harness
            .client
            .put("/auth/mfa/ticket")
            .header(Header::new("X-Session-Token", session.token))
            .body(serde_json::json!({ "google_id_token": "anything" }).to_string())
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::BadRequest);
        assert!(matches!(
            res.into_json::<Error>().await.unwrap().error_type,
            ErrorType::DisallowedMFAMethod,
        ));
    }
}
