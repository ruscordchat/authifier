/// Logout of current session
/// POST /session/logout
use rocket::State;

use crate::entities::*;
use crate::logic::Auth;
use crate::util::{EmptyResponse, Error, Result};

#[post("/logout")]
pub async fn logout(auth: &State<Auth>, session: Session) -> Result<EmptyResponse> {
    session
        .delete(&auth.db)
        .await
        .map(|_| EmptyResponse)
        .map_err(|_| Error::DatabaseError {
            operation: "delete",
            with: "session",
        })
}

#[cfg(test)]
mod tests {
    use crate::test::*;

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn success() {
        use rocket::http::Header;

        let (_, auth) = for_test("logout::success").await;

        let account = auth
            .create_account("email@example.com".into(), "password".into(), false)
            .await
            .unwrap();

        let session = auth
            .create_session(&account, "my session".into())
            .await
            .unwrap();

        let client =
            bootstrap_rocket_with_auth(auth, routes![crate::web::session::logout::logout]).await;

        let res = client
            .post("/logout")
            .header(Header::new("X-Session-Token", session.token))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::NoContent);
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_invalid_session() {
        use rocket::http::Header;

        let client = bootstrap_rocket(
            "logout",
            "fail_invalid_session",
            routes![crate::web::session::logout::logout],
        )
        .await;

        let res = client
            .post("/logout")
            .header(Header::new("X-Session-Token", "invalid"))
            .dispatch()
            .await;

        assert_eq!(res.status(), Status::Unauthorized);
    }

    #[cfg(feature = "async-std-runtime")]
    #[async_std::test]
    async fn fail_no_session() {
        let client = bootstrap_rocket(
            "logout",
            "fail_no_session",
            routes![crate::web::session::logout::logout],
        )
        .await;

        let res = client.post("/logout").dispatch().await;

        assert_eq!(res.status(), Status::BadRequest);
    }
}