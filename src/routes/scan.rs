use chrono::Utc;
use rocket::form::Form;
use rocket::http::Status;
use rocket::response::Redirect;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};

use crate::models::{Group, Post, Scan};
use crate::AppState;

#[derive(FromForm)]
pub struct ScanForm {
    post_id: String,
}

#[get("/<group_id>")]
pub fn scan_page(state: &State<AppState>, group_id: &str) -> Result<Template, Status> {
    let db = state.db.lock().unwrap();

    let group = match Group::get_by_id(db.conn(), group_id).ok().flatten() {
        Some(g) => g,
        None => return Err(Status::BadRequest),
    };

    let posts = Post::get_all(db.conn()).unwrap_or_default();
    let scans = Scan::get_by_group(db.conn(), group_id).unwrap_or_default();

    Ok(Template::render(
        "scan",
        context! {
            group: group,
            posts: posts,
            scans: scans,
        },
    ))
}

#[post("/<group_id>", data = "<form>")]
pub fn record_scan(state: &State<AppState>, group_id: &str, form: Form<ScanForm>) -> Redirect {
    let db = state.db.lock().unwrap();

    // Verify group exists
    if Group::get_by_id(db.conn(), group_id)
        .ok()
        .flatten()
        .is_none()
    {
        return Redirect::to("/");
    }

    // Regular post: toggle arrival/departure
    match Scan::get_by_group_and_post(db.conn(), group_id, &form.post_id)
        .ok()
        .flatten()
    {
        Some(scan) => {
            // Already arrived, record departure
            if scan.departure_time.is_none() {
                let _ = Scan::set_departure_time(db.conn(), &scan.id, Utc::now());
            }
        }
        None => {
            // First scan, record arrival
            let scan = Scan::new(group_id.to_string(), form.post_id.clone());
            let _ = scan.insert(db.conn());
        }
    }

    Redirect::to(format!("/scan/{}", group_id))
}

pub fn routes() -> Vec<Route> {
    routes![scan_page, record_scan]
}
