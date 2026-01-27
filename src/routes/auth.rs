use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};

use crate::auth::{self, Admin};

#[derive(FromForm)]
pub struct LoginForm {
    password: String,
}

#[get("/login")]
pub fn login_page(_admin: Admin) -> Redirect {
    // Already logged in
    Redirect::to("/admin/groups")
}

#[get("/login", rank = 2)]
pub fn login_form() -> Template {
    Template::render("login", context! { is_admin: false })
}

#[post("/login", data = "<form>")]
pub fn login(cookies: &CookieJar<'_>, form: Form<LoginForm>) -> Result<Redirect, Template> {
    if auth::check_password(&form.password) {
        auth::login(cookies);
        Ok(Redirect::to("/admin/groups"))
    } else {
        Err(Template::render(
            "login",
            context! { error: "Invalid password", is_admin: false },
        ))
    }
}

#[get("/logout")]
pub fn logout(cookies: &CookieJar<'_>) -> Redirect {
    auth::logout(cookies);
    Redirect::to("/")
}

pub fn routes() -> Vec<Route> {
    routes![login_page, login_form, login, logout]
}
