use chrono::Utc;
use image::{ImageEncoder, Luma};
use qrcode::QrCode;
use rocket::form::Form;
use rocket::http::ContentType;
use rocket::response::Redirect;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::models::Group;
use crate::AppState;

#[derive(FromForm)]
pub struct NewGroup {
    name: String,
}

#[get("/")]
pub fn groups(state: &State<AppState>) -> Template {
    let db = state.db.lock().unwrap();
    let groups = Group::get_all(db.conn()).unwrap_or_default();
    Template::render("admin/groups", context! { groups: groups })
}

#[post("/", data = "<form>")]
pub fn create_group(state: &State<AppState>, form: Form<NewGroup>) -> Redirect {
    let db = state.db.lock().unwrap();

    let group = Group::new(form.name.clone());
    group.insert(db.conn()).unwrap();

    Redirect::to("/admin/groups")
}

#[get("/<id>/delete")]
pub fn delete_group(state: &State<AppState>, id: &str) -> Redirect {
    let db = state.db.lock().unwrap();
    let _ = Group::delete(db.conn(), id);
    Redirect::to("/admin/groups")
}

#[get("/<id>/start_timer")]
pub fn start_group_timer(state: &State<AppState>, id: &str) -> Redirect {
    let db = state.db.lock().unwrap();
    let _ = Group::set_start_time(db.conn(), id, Utc::now());
    Redirect::to("/admin/groups")
}

#[get("/<id>/stop_timer")]
pub fn stop_group_timer(state: &State<AppState>, id: &str) -> Redirect {
    let db = state.db.lock().unwrap();
    let _ = Group::set_finish_time(db.conn(), id, Utc::now());
    Redirect::to("/admin/groups")
}

#[get("/<id>/qr")]
pub fn group_qr(_state: &State<AppState>, id: &str) -> (ContentType, Vec<u8>) {
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
