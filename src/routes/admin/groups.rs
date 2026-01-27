use chrono::Utc;
use image::{ImageEncoder, Luma};
use qrcode::QrCode;
use rocket::form::Form;
use rocket::http::ContentType;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};

use crate::auth::Admin;
use crate::db::DbConn;
use crate::models::{Group, NewGroup};

#[derive(FromForm)]
pub struct NewGroupForm {
    name: String,
    scout_group: String,
    members: String,
    phone_number: String,
}

#[get("/")]
pub async fn groups(_admin: Admin, conn: DbConn) -> Template {
    let groups = conn.run(|c| Group::get_all(c)).await.unwrap_or_default();
    Template::render("admin/groups", context! { groups: groups, is_admin: true })
}

#[post("/", data = "<form>")]
pub async fn create_group(_admin: Admin, conn: DbConn, form: Form<NewGroupForm>) -> Redirect {
    let name = form.name.clone();
    let scout_group = form.scout_group.clone();
    let members = form.members.clone();
    let phone_number = form.phone_number.clone();
    let result = conn
        .run(move |c| {
            let group = NewGroup::new(name, scout_group, members, phone_number);
            Group::insert(c, group)
        })
        .await;
    if let Err(e) = result {
        eprintln!("Failed to create group: {}", e);
    }

    Redirect::to("/admin/groups")
}

#[get("/<id>/delete")]
pub async fn delete_group(_admin: Admin, conn: DbConn, id: String) -> Redirect {
    conn.run(move |c| Group::delete(c, &id)).await.ok();
    Redirect::to("/admin/groups")
}

#[get("/<id>/start_timer")]
pub async fn start_group_timer(_admin: Admin, conn: DbConn, id: String) -> Redirect {
    let now = Utc::now().naive_utc();
    conn.run(move |c| Group::set_start_time(c, &id, now))
        .await
        .ok();
    Redirect::to("/admin/groups")
}

#[get("/<id>/stop_timer")]
pub async fn stop_group_timer(_admin: Admin, conn: DbConn, id: String) -> Redirect {
    let now = Utc::now().naive_utc();
    conn.run(move |c| Group::set_finish_time(c, &id, now))
        .await
        .ok();
    Redirect::to("/admin/groups")
}

#[get("/<id>/qr")]
pub fn group_qr(_admin: Admin, id: &str) -> (ContentType, Vec<u8>) {
    let url = format!("/scan/{}", id);

    let code = QrCode::new(url.as_bytes()).unwrap();
    let image = code.render::<Luma<u8>>().min_dimensions(200, 200).build();

    let mut png_data: Vec<u8> = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut png_data);
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::L8,
        )
        .unwrap();

    (ContentType::PNG, png_data)
}

pub fn routes() -> Vec<Route> {
    routes![
        groups,
        create_group,
        start_group_timer,
        stop_group_timer,
        delete_group,
        group_qr
    ]
}
