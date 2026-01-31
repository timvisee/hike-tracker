use rocket::http::{Cookie, CookieJar, Status};
use rocket::request::{FromRequest, Outcome, Request};
use rocket::time::Duration;
use serde::{Deserialize, Serialize};

const AUTH_COOKIE: &str = "auth_session";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthSession {
    Admin,
    PostHolder { post_id: String },
}

pub struct Admin;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Admin {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_auth_session(request.cookies()) {
            Some(AuthSession::Admin) => Outcome::Success(Admin),
            _ => Outcome::Forward(Status::Unauthorized),
        }
    }
}

pub struct AnyAuth {
    pub is_admin: bool,
    pub post_id: Option<String>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AnyAuth {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match get_auth_session(request.cookies()) {
            Some(AuthSession::Admin) => Outcome::Success(AnyAuth {
                is_admin: true,
                post_id: None,
            }),
            Some(AuthSession::PostHolder { post_id }) => Outcome::Success(AnyAuth {
                is_admin: false,
                post_id: Some(post_id),
            }),
            None => Outcome::Forward(Status::Unauthorized),
        }
    }
}

pub struct CurrentPath(pub String);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for CurrentPath {
    type Error = ();
    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, ()> {
        Outcome::Success(CurrentPath(req.uri().path().to_string()))
    }
}

fn get_auth_session(cookies: &CookieJar<'_>) -> Option<AuthSession> {
    cookies
        .get_private(AUTH_COOKIE)
        .and_then(|cookie| serde_json::from_str(cookie.value()).ok())
}

pub fn login_admin(cookies: &CookieJar<'_>) {
    let session = AuthSession::Admin;
    let value = serde_json::to_string(&session).expect("Failed to serialize session");
    let mut cookie = Cookie::new(AUTH_COOKIE, value);
    cookie.set_max_age(Duration::hours(24));
    cookies.add_private(cookie);
}

pub fn login_post_holder(cookies: &CookieJar<'_>, post_id: String) {
    let session = AuthSession::PostHolder { post_id };
    let value = serde_json::to_string(&session).expect("Failed to serialize session");
    let mut cookie = Cookie::new(AUTH_COOKIE, value);
    cookie.set_max_age(Duration::hours(24));
    cookies.add_private(cookie);
}

pub fn logout(cookies: &CookieJar<'_>) {
    cookies.remove_private(AUTH_COOKIE);
}

pub fn check_admin_password(password: &str) -> bool {
    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_default();
    !admin_password.is_empty() && password == admin_password
}

pub fn is_admin(cookies: &CookieJar<'_>) -> bool {
    matches!(get_auth_session(cookies), Some(AuthSession::Admin))
}

pub fn get_current_auth(cookies: &CookieJar<'_>) -> Option<AuthSession> {
    get_auth_session(cookies)
}

pub struct AuthContext {
    pub is_admin: bool,
    pub is_post_holder: bool,
    pub holder_post_id: Option<String>,
}

pub fn get_auth_context(cookies: &CookieJar<'_>) -> AuthContext {
    let current_auth = get_auth_session(cookies);
    let is_admin = matches!(&current_auth, Some(AuthSession::Admin));
    let holder_post_id = match &current_auth {
        Some(AuthSession::PostHolder { post_id }) => Some(post_id.clone()),
        _ => None,
    };
    let is_post_holder = holder_post_id.is_some();

    AuthContext {
        is_admin,
        is_post_holder,
        holder_post_id,
    }
}
