use chrono::{NaiveDateTime, Utc};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};

use crate::auth::{self, Admin, AnyAuth};
use crate::db::DbConn;
use crate::models::{Group, NewScan, Post, Scan};

use super::get_scout_groups;

#[get("/<group_id>/edit")]
pub async fn edit_page(
    auth: AnyAuth,
    cookies: &CookieJar<'_>,
    conn: DbConn,
    group_id: String,
) -> Result<Template, Redirect> {
    let is_admin = auth::is_admin(cookies);
    let gid = group_id.clone();
    let group = conn
        .run(move |c| Group::get_by_id(c, &gid))
        .await
        .ok()
        .flatten();

    let group = match group {
        Some(g) => g,
        None => return Err(Redirect::to("/dashboard")),
    };

    let gid = group_id.clone();
    let posts = conn.run(Post::get_all).await.unwrap_or_default();
    let scans = conn
        .run(move |c| Scan::get_by_group(c, &gid))
        .await
        .unwrap_or_default();

    let scout_groups = get_scout_groups();

    // For post holders, filter to only show their post
    let filtered_posts = if let Some(ref holder_post_id) = auth.post_id {
        posts
            .into_iter()
            .filter(|p| &p.id == holder_post_id)
            .collect()
    } else {
        posts
    };

    let is_post_holder = auth.post_id.is_some();

    Ok(Template::render(
        "scan_edit",
        context! {
            group: group,
            posts: filtered_posts,
            scans: scans,
            is_admin: is_admin,
            scout_groups: scout_groups,
            holder_post_id: auth.post_id,
            is_post_holder: is_post_holder,
        },
    ))
}

#[derive(FromForm)]
pub struct UpdateScanForm {
    arrival_time: String,
    departure_time: Option<String>,
    clear_departure: Option<String>,
}

#[post("/<group_id>/edit/scan/<scan_id>/update", data = "<form>")]
pub async fn update_scan(
    auth: AnyAuth,
    conn: DbConn,
    group_id: String,
    scan_id: String,
    form: Form<UpdateScanForm>,
) -> Redirect {
    // For post holders, verify they can only edit scans for their post
    if let Some(ref holder_post_id) = auth.post_id {
        let sid = scan_id.clone();
        let scan = conn
            .run(move |c| Scan::get_by_id(c, &sid))
            .await
            .ok()
            .flatten();

        if let Some(scan) = scan {
            if &scan.post_id != holder_post_id {
                return Redirect::to(format!("/scan/{group_id}/edit"));
            }
        }
    }

    // Parse arrival time
    if let Ok(arrival) = NaiveDateTime::parse_from_str(&form.arrival_time, "%Y-%m-%dT%H:%M") {
        let sid = scan_id.clone();
        conn.run(move |c| Scan::set_arrival_time(c, &sid, arrival))
            .await
            .ok();
    }

    // Check if clear checkbox is checked
    if form.clear_departure.is_some() {
        let sid = scan_id.clone();
        conn.run(move |c| Scan::clear_departure_time(c, &sid))
            .await
            .ok();
    } else if let Some(dt) = &form.departure_time {
        if !dt.is_empty() {
            if let Ok(departure) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M") {
                let sid = scan_id.clone();
                conn.run(move |c| Scan::set_departure_time(c, &sid, departure))
                    .await
                    .ok();
            }
        }
    }

    Redirect::to(format!("/scan/{group_id}/edit"))
}

#[get("/<group_id>/edit/scan/<scan_id>/delete")]
pub async fn delete_scan(
    auth: AnyAuth,
    conn: DbConn,
    group_id: String,
    scan_id: String,
) -> Redirect {
    // For post holders, verify they can only delete scans for their post
    if let Some(ref holder_post_id) = auth.post_id {
        let sid = scan_id.clone();
        let scan = conn
            .run(move |c| Scan::get_by_id(c, &sid))
            .await
            .ok()
            .flatten();

        if let Some(scan) = scan {
            if &scan.post_id != holder_post_id {
                return Redirect::to(format!("/scan/{group_id}/edit"));
            }
        }
    }

    conn.run(move |c| Scan::delete(c, &scan_id)).await.ok();
    Redirect::to(format!("/scan/{group_id}/edit"))
}

#[derive(FromForm)]
pub struct AddScanForm {
    post_id: String,
    arrival_time: String,
    departure_time: Option<String>,
}

#[post("/<group_id>/edit/scan/add", data = "<form>")]
pub async fn add_scan(
    auth: AnyAuth,
    conn: DbConn,
    group_id: String,
    form: Form<AddScanForm>,
) -> Redirect {
    // Post holders can only add scans for their assigned post
    if let Some(ref holder_post_id) = auth.post_id {
        if &form.post_id != holder_post_id {
            return Redirect::to(format!("/scan/{group_id}/edit"));
        }
    }

    if let Ok(arrival) = NaiveDateTime::parse_from_str(&form.arrival_time, "%Y-%m-%dT%H:%M") {
        let gid = group_id.clone();
        let post_id = form.post_id.clone();
        let departure = form
            .departure_time
            .as_ref()
            .filter(|s| !s.is_empty())
            .and_then(|dt| NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M").ok());

        conn.run(move |c| {
            let scan = NewScan {
                id: uuid::Uuid::new_v4().to_string(),
                group_id: gid,
                post_id,
                arrival_time: arrival,
                departure_time: departure,
            };
            Scan::insert(c, scan)
        })
        .await
        .ok();
    }

    Redirect::to(format!("/scan/{group_id}/edit"))
}

#[derive(FromForm)]
pub struct UpdateGroupForm {
    start_time: Option<String>,
    finish_time: Option<String>,
    clear_start: Option<String>,
    clear_finish: Option<String>,
}

#[post("/<group_id>/edit/group/update", data = "<form>")]
pub async fn update_group(
    _admin: Admin, // Group timer edits are admin-only
    conn: DbConn,
    group_id: String,
    form: Form<UpdateGroupForm>,
) -> Redirect {
    // Handle start time
    if form.clear_start.is_some() {
        let gid = group_id.clone();
        conn.run(move |c| Group::clear_start_time(c, &gid))
            .await
            .ok();
    } else if let Some(dt) = &form.start_time {
        if !dt.is_empty() {
            if let Ok(start) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M") {
                let gid = group_id.clone();
                conn.run(move |c| Group::set_start_time(c, &gid, start))
                    .await
                    .ok();
            }
        }
    }

    // Handle finish time
    if form.clear_finish.is_some() {
        let gid = group_id.clone();
        conn.run(move |c| Group::clear_finish_time(c, &gid))
            .await
            .ok();
    } else if let Some(dt) = &form.finish_time {
        if !dt.is_empty() {
            if let Ok(finish) = NaiveDateTime::parse_from_str(dt, "%Y-%m-%dT%H:%M") {
                let gid = group_id.clone();
                conn.run(move |c| Group::set_finish_time(c, &gid, finish))
                    .await
                    .ok();
            }
        }
    }

    Redirect::to(format!("/scan/{group_id}/edit"))
}

#[derive(FromForm)]
pub struct UpdateGroupDetailsForm {
    name: String,
    scout_group: String,
    members: String,
    phone_number: String,
    group_number: i32,
    route: String,
    start_timer: Option<String>,
}

#[post("/<group_id>/edit/group/details", data = "<form>")]
pub async fn update_group_details(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    group_id: String,
    form: Form<UpdateGroupDetailsForm>,
) -> Redirect {
    let is_admin = auth::is_admin(cookies);

    // Check if group exists and whether it has started
    let gid = group_id.clone();
    let group = conn
        .run(move |c| Group::get_by_id(c, &gid))
        .await
        .ok()
        .flatten();

    let group = match group {
        Some(g) => g,
        None => return Redirect::to(format!("/scan/{group_id}")),
    };

    // Only admin can edit details of started groups
    if group.start_time.is_some() && !is_admin {
        return Redirect::to(format!("/scan/{group_id}"));
    }

    let gid = group_id.clone();
    let name = form.name.clone();
    let scout_group = form.scout_group.clone();
    let members = form.members.clone();
    let phone_number = form.phone_number.clone();
    let group_number = form.group_number;
    let route = form.route.clone();

    conn.run(move |c| {
        Group::update_details(
            c,
            &gid,
            &name,
            &scout_group,
            &members,
            &phone_number,
            group_number,
            &route,
        )
    })
    .await
    .ok();

    // If start_timer was requested (admin only), start the timer
    if form.start_timer.is_some() && is_admin {
        let gid = group_id.clone();
        let now = Utc::now().naive_utc();
        conn.run(move |c| Group::set_start_time(c, &gid, now))
            .await
            .ok();
        return Redirect::to(format!("/scan/{group_id}"));
    }

    // For unstarted groups, redirect back to the scan page (shows the edit form again)
    // For started groups (admin), redirect to the edit page
    if group.start_time.is_none() {
        Redirect::to(format!("/scan/{group_id}"))
    } else {
        Redirect::to(format!("/scan/{group_id}/edit"))
    }
}

pub fn routes() -> Vec<Route> {
    routes![
        edit_page,
        update_scan,
        delete_scan,
        add_scan,
        update_group,
        update_group_details
    ]
}
