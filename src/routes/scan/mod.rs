pub mod edit;

use chrono::Utc;
use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::response::Redirect;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::auth::{get_auth_context, AnyAuth, CurrentPath};
use crate::db::DbConn;
use crate::models::{Group, NewGroup, NewScan, Post, Scan};
use crate::stats::calculate_group_stats;

#[derive(FromForm)]
pub struct ScanForm {
    action: String,
}

#[derive(Serialize)]
pub struct NextAction {
    pub action_id: String,
    pub label: String,
}

pub fn get_scout_groups() -> Vec<String> {
    std::env::var("SCOUT_GROUPS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
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
                    label: format!("Aankomst bij Post {}: {}", post.post_order, post.name),
                });
            }
            Some(s) if s.departure_time.is_none() => {
                // At this post, next action is leave
                return Some(NextAction {
                    action_id: format!("LEAVE_{}", post.id),
                    label: format!("Vertrek van Post {}: {}", post.post_order, post.name),
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
pub async fn scan_page(
    cookies: &CookieJar<'_>,
    conn: DbConn,
    group_id: String,
    path: CurrentPath,
) -> Template {
    let auth_ctx = get_auth_context(cookies);
    let is_admin = auth_ctx.is_admin;
    let is_post_holder = auth_ctx.is_post_holder;
    let holder_post_id = auth_ctx.holder_post_id;

    let gid = group_id.clone();
    let group = conn
        .run(move |c| Group::get_by_id(c, &gid))
        .await
        .ok()
        .flatten();

    let scout_groups = get_scout_groups();

    let group = match group {
        Some(g) => g,
        None => {
            return Template::render(
                "scan_new_group",
                context! {
                    group_id: group_id,
                    is_admin: is_admin,
                    is_post_holder: is_post_holder,
                    holder_post_id: holder_post_id,
                    scout_groups: scout_groups,
                },
            );
        }
    };

    // If group hasn't started yet, show the edit form (same as new group form but pre-filled)
    if group.start_time.is_none() {
        return Template::render(
            "scan_new_group",
            context! {
                group_id: group_id,
                group: group,
                is_admin: is_admin,
                is_post_holder: is_post_holder,
                holder_post_id: holder_post_id,
                scout_groups: scout_groups,
                is_existing: true,
            },
        );
    }

    let gid = group_id.clone();
    let posts = conn.run(Post::get_all).await.unwrap_or_default();
    let scans = conn
        .run(move |c| Scan::get_by_group(c, &gid))
        .await
        .unwrap_or_default();

    let next_action = get_next_action(&group, &posts, &scans);
    let stats = calculate_group_stats(&group, &scans, posts.clone());
    let emergency_info = std::env::var("EMERGENCY_INFO").ok();

    Template::render(
        "scan",
        context! {
            group: group,
            posts: posts,
            scans: scans,
            is_admin: is_admin,
            is_post_holder: is_post_holder,
            holder_post_id: holder_post_id,
            next_action: next_action,
            stats: stats,
            emergency_info: emergency_info,
            current_path: path.0
        },
    )
}

#[post("/<group_id>", data = "<form>")]
pub async fn record_scan(
    auth: AnyAuth,
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

    // Handle start timer (admin only)
    if action == "__START_TIMER__" {
        if !auth.is_admin {
            return Redirect::to(format!("/scan/{group_id}"));
        }
        let gid = group_id.clone();
        let now = Utc::now().naive_utc();
        conn.run(move |c| Group::set_start_time(c, &gid, now))
            .await
            .ok();
        return Redirect::to(format!("/scan/{group_id}"));
    }

    // Handle stop timer (admin only)
    if action == "__STOP_TIMER__" {
        if !auth.is_admin {
            return Redirect::to(format!("/scan/{group_id}"));
        }
        let gid = group_id.clone();
        let now = Utc::now().naive_utc();
        conn.run(move |c| Group::set_finish_time(c, &gid, now))
            .await
            .ok();
        return Redirect::to(format!("/scan/{group_id}"));
    }

    // Handle arrive at post
    if let Some(post_id) = action.strip_prefix("ARRIVE_") {
        // Post holders can only scan for their assigned post
        if let Some(ref holder_post_id) = auth.post_id {
            if holder_post_id != post_id {
                return Redirect::to(format!("/scan/{group_id}"));
            }
        }
        let gid = group_id.clone();
        let post_id = post_id.to_string();
        conn.run(move |c| {
            let scan = NewScan::new(gid, post_id);
            Scan::insert(c, scan)
        })
        .await
        .ok();
        return Redirect::to(format!("/scan/{group_id}"));
    }

    // Handle leave post
    if let Some(post_id) = action.strip_prefix("LEAVE_") {
        // Post holders can only scan for their assigned post
        if let Some(ref holder_post_id) = auth.post_id {
            if holder_post_id != post_id {
                return Redirect::to(format!("/scan/{group_id}"));
            }
        }
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
        return Redirect::to(format!("/scan/{group_id}"));
    }

    Redirect::to(format!("/scan/{group_id}"))
}

#[derive(FromForm)]
pub struct NewGroupForm {
    name: String,
    scout_group: String,
    members: String,
    phone_number: String,
    group_number: i32,
    route: String,
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
    let group_number = form.group_number;
    let route = form.route.clone();
    let gid = group_id.clone();
    let result = conn
        .run(move |c| {
            let group = NewGroup::new_with_id(
                gid,
                name,
                scout_group,
                members,
                phone_number,
                group_number,
                route,
            );
            Group::insert(c, group)
        })
        .await;
    if let Err(err) = result {
        eprintln!("Failed to create group from scan: {err}");
    }
    Redirect::to(format!("/scan/{group_id}"))
}

pub fn routes() -> Vec<Route> {
    routes![scan_page, record_scan, create_group_from_scan]
}
