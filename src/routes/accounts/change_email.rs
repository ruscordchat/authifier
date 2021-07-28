use crate::util::Error;
use crate::{
    auth::{Auth, Session},
    options::EmailVerification,
};

use chrono::Utc;
use mongodb::bson::{doc, Bson};
use nanoid::nanoid;
use rocket::State;
use rocket_contrib::json::Json;
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct ChangeEmail {
    #[validate(length(min = 8, max = 1024))]
    password: String,
    #[validate(email)]
    new_email: String,
}

#[post("/change/email", data = "<data>")]
pub async fn change_email(
    auth: State<'_, Auth>,
    session: Session,
    data: Json<ChangeEmail>,
) -> crate::util::Result<()> {
    data.validate()
        .map_err(|error| Error::FailedValidation { error })?;

    auth.verify_password(&session, data.password.clone())
        .await?;
    let normalised = auth.check_email_is_use(data.new_email.clone()).await?;

    let set = if let EmailVerification::Enabled {
        smtp,
        templates,
        verification_expiry,
        ..
    } = &auth.options.email_verification
    {
        let token = nanoid!(32);
        auth.email_send_verification(&smtp, &templates, &data.new_email, &token)?;

        doc! {
            "verification": {
                "status": "Moving",
                "new_email": &data.new_email,
                "token": token,
                "expiry": Bson::DateTime(Utc::now() + *verification_expiry)
            }
        }
    } else {
        doc! {
            "verification": {
                "status": "Verified"
            },
            "email": &data.new_email,
            "email_normalised": normalised
        }
    };

    auth.collection
        .update_one(
            doc! {
                "_id": &session.user_id,
            },
            doc! {
                "$set": set
            },
            None,
        )
        .await
        .map_err(|_| Error::DatabaseError {
            operation: "update_one",
            with: "accounts",
        })?;

    Ok(())
}