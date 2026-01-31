use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};

use crate::auth::{self, Admin, AuthSession};
use crate::db::DbConn;
use crate::models::Post;

#[derive(FromForm)]
pub struct LoginForm {
    password: String,
    next: String,
}

#[get("/login")]
pub fn login_page(_admin: Admin) -> Redirect {
    // Already logged in as admin
    Redirect::to("/admin/groups")
}

#[get("/login?<next>", rank = 2)]
pub fn login_form(cookies: &CookieJar<'_>, next: Option<String>) -> Result<Redirect, Template> {
    // Check if logged in as post holder
    if let Some(AuthSession::PostHolder { post_id }) = auth::get_current_auth(cookies) {
        return Ok(Redirect::to(format!("/post/{post_id}")));
    }
    Err(Template::render(
        "login",
        context! { is_admin: false, next: next },
    ))
}

#[post("/login", data = "<form>")]
pub async fn login(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    form: Form<LoginForm>,
) -> Result<Redirect, Template> {
    let password = form.password.clone();
    let next = if form.next.starts_with('/') {
        form.next.clone()
    } else {
        "/".to_string()
    };

    // First, check if it's the admin password
    if auth::check_admin_password(&password) {
        auth::login_admin(cookies);
        return Ok(Redirect::to(next));
    }

    // Then, check if it matches any post's password
    let post = conn
        .run(move |c| Post::find_by_password(c, &password))
        .await
        .ok()
        .flatten();

    if let Some(post) = post {
        auth::login_post_holder(cookies, post.id.clone());
        return Ok(Redirect::to(next));
    }

    // Invalid password
    Err(Template::render(
        "login",
        context! { error: "Ongeldig wachtwoord", is_admin: false },
    ))
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    auth::logout(cookies);
    Redirect::to("/")
}

pub fn routes() -> Vec<Route> {
    routes![login_page, login_form, login, logout]
}
