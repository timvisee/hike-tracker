use chrono::{NaiveDateTime, TimeDelta, Utc};
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::auth::{self, Admin};
use crate::db::DbConn;
use crate::models::{Group, NewGroup, NewScan, Post, Scan};

#[derive(FromForm)]
pub struct ScanForm {
    action: String,
}

#[derive(Serialize)]
pub struct NextAction {
    pub action_id: String,
    pub label: String,
}

#[derive(Serialize)]
pub struct PostScanInfo {
    pub post: Post,
    pub scan: Option<Scan>,
    pub idle_time: Option<TimeDelta>,
}

#[derive(Serialize)]
pub struct GroupStats {
    pub total_time: Option<TimeDelta>,
    pub walking_time: Option<TimeDelta>,
    pub idle_time: TimeDelta,
    pub post_scans: Vec<PostScanInfo>,
}

fn now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}

fn calculate_stats(group: &Group, posts: &[Post], scans: &[Scan]) -> GroupStats {
    let post_scans: Vec<PostScanInfo> = posts
        .iter()
        .map(|post| {
            let scan = scans.iter().find(|s| s.post_id == post.id).cloned();
            let idle_time = scan
                .as_ref()
                .map(|s| s.departure_time.unwrap_or_else(now_naive) - s.arrival_time);
            PostScanInfo {
                post: post.clone(),
                scan,
                idle_time,
            }
        })
        .collect();

    let idle_time: TimeDelta = post_scans.iter().filter_map(|ps| ps.idle_time).sum();

    let total_time = group
        .start_time
        .map(|start| group.finish_time.unwrap_or_else(now_naive) - start);

    let walking_time = total_time.map(|t| t - idle_time);

    GroupStats {
        total_time,
        walking_time,
        idle_time,
        post_scans,
    }
}

fn get_next_action(group: &Group, posts: &[Post], scans: &[Scan]) -> Option<NextAction> {
    // If group is finished, no next action
    if group.finish_time.is_some() {
        return None;
    }

    // If not started, next action is start timer
    if group.start_time.is_none() {
        return Some(NextAction {
            action_id: "__START_TIMER__".to_string(),
            label: "Start Timer".to_string(),
        });
    }

    // Check each post in order
    for post in posts {
        let scan = scans.iter().find(|s| s.post_id == post.id);
        match scan {
            None => {
                // No scan for this post, next action is arrive
                return Some(NextAction {
                    action_id: format!("ARRIVE_{}", post.id),
                    label: format!("Arrive at {} {}", post.post_order, post.name),
                });
            }
            Some(s) if s.departure_time.is_none() => {
                // At this post, next action is leave
                return Some(NextAction {
                    action_id: format!("LEAVE_{}", post.id),
                    label: format!("Leave {} {}", post.post_order, post.name),
                });
            }
            Some(_) => {
                // Completed this post, check next
                continue;
            }
        }
    }

    // All posts completed, next action is stop timer
    Some(NextAction {
        action_id: "__STOP_TIMER__".to_string(),
        label: "Stop Timer".to_string(),
    })
}

#[get("/<group_id>")]
pub async fn scan_page(cookies: &CookieJar<'_>, conn: DbConn, group_id: String) -> Template {
    let is_admin = auth::is_admin(cookies);
    let gid = group_id.clone();
    let group = conn
        .run(move |c| Group::get_by_id(c, &gid))
        .await
        .ok()
        .flatten();

    let group = match group {
        Some(g) => g,
        None => {
            return Template::render(
                "scan_new_group",
                context! { group_id: group_id, is_admin: is_admin },
            );
        }
    };

    let gid = group_id.clone();
    let posts = conn.run(|c| Post::get_all(c)).await.unwrap_or_default();
    let scans = conn
        .run(move |c| Scan::get_by_group(c, &gid))
        .await
        .unwrap_or_default();

    let next_action = get_next_action(&group, &posts, &scans);
    let stats = calculate_stats(&group, &posts, &scans);

    Template::render(
        "scan",
        context! {
            group: group,
            posts: posts,
            scans: scans,
            is_admin: is_admin,
            next_action: next_action,
            stats: stats,
        },
    )
}

#[post("/<group_id>", data = "<form>")]
pub async fn record_scan(
    _admin: Admin,
    conn: DbConn,
    group_id: String,
    form: Form<ScanForm>,
) -> Redirect {
    let gid = group_id.clone();

    // Verify group exists
    let group_exists = conn
        .run(move |c| Group::get_by_id(c, &gid))
        .await
        .ok()
        .flatten()
        .is_some();

    if !group_exists {
        return Redirect::to("/");
    }

    let action = form.action.clone();

    // Handle start timer
    if action == "__START_TIMER__" {
        let gid = group_id.clone();
        let now = Utc::now().naive_utc();
        conn.run(move |c| Group::set_start_time(c, &gid, now))
            .await
            .ok();
        return Redirect::to(format!("/scan/{}", group_id));
    }

    // Handle stop timer
    if action == "__STOP_TIMER__" {
        let gid = group_id.clone();
        let now = Utc::now().naive_utc();
        conn.run(move |c| Group::set_finish_time(c, &gid, now))
            .await
            .ok();
        return Redirect::to(format!("/scan/{}", group_id));
    }

    // Handle arrive at post
    if let Some(post_id) = action.strip_prefix("ARRIVE_") {
        let gid = group_id.clone();
        let post_id = post_id.to_string();
        conn.run(move |c| {
            let scan = NewScan::new(gid, post_id);
            Scan::insert(c, scan)
        })
        .await
        .ok();
        return Redirect::to(format!("/scan/{}", group_id));
    }

    // Handle leave post
    if let Some(post_id) = action.strip_prefix("LEAVE_") {
        let gid = group_id.clone();
        let post_id = post_id.to_string();
        let existing_scan = conn
            .run(move |c| Scan::get_by_group_and_post(c, &gid, &post_id))
            .await
            .ok()
            .flatten();

        if let Some(scan) = existing_scan {
            if scan.departure_time.is_none() {
                let scan_id = scan.id.clone();
                let now = Utc::now().naive_utc();
                conn.run(move |c| Scan::set_departure_time(c, &scan_id, now))
                    .await
                    .ok();
            }
        }
        return Redirect::to(format!("/scan/{}", group_id));
    }

    Redirect::to(format!("/scan/{}", group_id))
}

#[derive(FromForm)]
pub struct NewGroupForm {
    name: String,
    scout_group: String,
    members: String,
    phone_number: String,
}

#[post("/<group_id>/create", data = "<form>")]
pub async fn create_group_from_scan(
    conn: DbConn,
    group_id: String,
    form: Form<NewGroupForm>,
) -> Redirect {
    let name = form.name.clone();
    let scout_group = form.scout_group.clone();
    let members = form.members.clone();
    let phone_number = form.phone_number.clone();
    let gid = group_id.clone();
    let result = conn
        .run(move |c| {
            let group = NewGroup::new_with_id(gid, name, scout_group, members, phone_number);
            Group::insert(c, group)
        })
        .await;
    if let Err(e) = result {
        eprintln!("Failed to create group from scan: {}", e);
    }
    Redirect::to(format!("/scan/{}", group_id))
}

// ============ EDIT PAGE ============

#[get("/<group_id>/edit")]
pub async fn edit_page(
    _admin: Admin,
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
    let posts = conn.run(|c| Post::get_all(c)).await.unwrap_or_default();
    let scans = conn
        .run(move |c| Scan::get_by_group(c, &gid))
        .await
        .unwrap_or_default();

    Ok(Template::render(
        "scan_edit",
        context! {
            group: group,
            posts: posts,
            scans: scans,
            is_admin: is_admin,
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
    _admin: Admin,
    conn: DbConn,
    group_id: String,
    scan_id: String,
    form: Form<UpdateScanForm>,
) -> Redirect {
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

    Redirect::to(format!("/scan/{}/edit", group_id))
}

#[get("/<group_id>/edit/scan/<scan_id>/delete")]
pub async fn delete_scan(
    _admin: Admin,
    conn: DbConn,
    group_id: String,
    scan_id: String,
) -> Redirect {
    conn.run(move |c| Scan::delete(c, &scan_id)).await.ok();
    Redirect::to(format!("/scan/{}/edit", group_id))
}

#[derive(FromForm)]
pub struct AddScanForm {
    post_id: String,
    arrival_time: String,
    departure_time: Option<String>,
}

#[post("/<group_id>/edit/scan/add", data = "<form>")]
pub async fn add_scan(
    _admin: Admin,
    conn: DbConn,
    group_id: String,
    form: Form<AddScanForm>,
) -> Redirect {
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

    Redirect::to(format!("/scan/{}/edit", group_id))
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
    _admin: Admin,
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

    Redirect::to(format!("/scan/{}/edit", group_id))
}

pub fn routes() -> Vec<Route> {
    routes![
        scan_page,
        record_scan,
        create_group_from_scan,
        edit_page,
        update_scan,
        delete_scan,
        add_scan,
        update_group
    ]
}
