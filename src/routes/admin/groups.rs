use image::{ImageEncoder, Luma};
use qrcode::QrCode;
use rocket::http::ContentType;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use uuid::Uuid;

use crate::auth::Admin;
use crate::db::DbConn;
use crate::models::Group;

#[get("/")]
pub async fn groups(_admin: Admin, conn: DbConn) -> Template {
    let groups = conn.run(Group::get_all).await.unwrap_or_default();
    Template::render("admin/groups", context! { groups: groups, is_admin: true })
}

#[get("/<id>/delete")]
pub async fn delete_group(_admin: Admin, conn: DbConn, id: String) -> Redirect {
    conn.run(move |c| Group::delete(c, &id)).await.ok();
    Redirect::to("/admin/groups")
}

#[get("/new")]
pub fn new_group(_admin: Admin) -> Redirect {
    let short_id = &Uuid::new_v4().to_string()[..8];
    Redirect::to(format!("/scan/{short_id}"))
}

#[get("/<id>/qr")]
pub fn group_qr(_admin: Admin, id: &str) -> (ContentType, Vec<u8>) {
    // TODO: Hardcoded url
    let url = format!("https://hike.qvdijk.nl/scan/{id}");

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
    routes![groups, new_group, delete_group, group_qr]
}
