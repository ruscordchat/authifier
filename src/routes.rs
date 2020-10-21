use rocket::{ Rocket, State };
use rocket::http::{Cookie, CookieJar};
use rocket_contrib::json::{ Json, JsonValue };
use super::auth::{ Auth, Create, Verify, Login, Session };

#[post("/create", data = "<data>")]
async fn create(auth: State<'_, Auth>, data: Json<Create>) -> super::util::Result<JsonValue> {
    Ok(json!({
        "user_id": auth.inner().create_account(data.into_inner()).await?
    }))
}

#[get("/verify/<code>")]
async fn verify(auth: State<'_, Auth>, code: String) -> super::util::Result<()> {
    auth.inner().verify_account(Verify { code }).await?;
    unimplemented!()
}

#[post("/login", data = "<data>")]
async fn login(auth: State<'_, Auth>, cookies: &CookieJar<'_>, data: Json<Login>) -> super::util::Result<JsonValue> {
    let session = auth.inner().login(data.into_inner()).await?;
    // ! FIXME: add a way to disable cookies
    cookies.add(Cookie::new("session_uid", session.user_id.clone()));
    cookies.add(Cookie::new("session_token", session.session_token.clone()));
    Ok(json!(session))
}

#[get("/check")]
async fn check(_session: Session) -> super::util::Result<()> {
    Ok(())
}

#[get("/sessions")]
async fn fetch_sessions(auth: State<'_, Auth>, session: Session) -> super::util::Result<JsonValue> {
    Ok(json!(auth.fetch_all_sessions(session).await?))
}

#[delete("/sessions/<id>")]
async fn deauth_session(auth: State<'_, Auth>, session: Session, id: String) -> super::util::Result<()> {
    auth.deauth_session(session, id).await
}

#[get("/logout")]
async fn logout(auth: State<'_, Auth>, cookies: &CookieJar<'_>, session: Session) -> super::util::Result<()> {
    let id = session.id.clone().unwrap();
    cookies.remove(Cookie::named("session_uid"));
    cookies.remove(Cookie::named("session_token"));
    auth.deauth_session(session, id).await
}

pub fn mount(rocket: Rocket, path: &str, auth: Auth) -> Rocket {
    rocket.manage(auth)
        .mount(path, routes![ create, verify, login, check, fetch_sessions, deauth_session, logout ])
}
