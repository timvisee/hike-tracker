use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::auth::is_admin;
use crate::db::DbConn;
use crate::models::{Group, Post, Scan};

#[derive(Serialize)]
struct GroupStatus {
    group: Group,
    arrival_time: String,
    departure_time: Option<String>,
}

#[get("/<post_id>")]
pub async fn post_overview(
    conn: DbConn,
    post_id: String,
    cookies: &rocket::http::CookieJar<'_>,
) -> Option<Template> {
    let post_id_clone = post_id.clone();
    let post = conn
        .run(move |c| Post::get_by_id(c, &post_id_clone))
        .await
        .ok()??;

    let post_id_clone = post_id.clone();
    let scans = conn
        .run(move |c| Scan::get_by_post(c, &post_id_clone))
        .await
        .unwrap_or_default();
    let all_groups = conn.run(|c| Group::get_all(c)).await.unwrap_or_default();
    let all_posts = conn.run(|c| Post::get_all(c)).await.unwrap_or_default();

    // Find this post's order to determine which groups should have arrived
    let post_order = post.post_order;

    // Get previous post (if any) to check if groups have left it
    let previous_post = all_posts.iter().find(|p| p.post_order < post_order);
    let previous_post_id = previous_post.map(|p| p.id.clone());

    // Get scans for previous post if it exists
    let prev_post_scans = if let Some(ref prev_id) = previous_post_id {
        let prev_id = prev_id.clone();
        conn.run(move |c| Scan::get_by_post(c, &prev_id))
            .await
            .unwrap_or_default()
    } else {
        vec![]
    };

    let mut groups_at_post = Vec::new();
    let mut groups_left = Vec::new();
    let mut groups_coming = Vec::new();

    for group in all_groups {
        // Skip groups that haven't started yet
        if group.start_time.is_none() {
            continue;
        }

        // Check if this group has a scan at this post
        let scan = scans.iter().find(|s| s.group_id == group.id);

        match scan {
            Some(s) => {
                let arrival_time = s.arrival_time.format("%H:%M:%S").to_string();
                let departure_time = s.departure_time.map(|dt| dt.format("%H:%M:%S").to_string());

                let status = GroupStatus {
                    group: group.clone(),
                    arrival_time,
                    departure_time: departure_time.clone(),
                };

                if s.departure_time.is_some() {
                    groups_left.push(status);
                } else {
                    groups_at_post.push(status);
                }
            }
            None => {
                // Group hasn't arrived at this post yet
                // Only show as "coming" if:
                // - This is the first post (no previous post), OR
                // - The group has left the previous post
                let should_show = if previous_post_id.is_none() {
                    // First post - all started groups that haven't arrived are coming
                    true
                } else {
                    // Check if group has left the previous post
                    prev_post_scans
                        .iter()
                        .any(|s| s.group_id == group.id && s.departure_time.is_some())
                };

                if should_show {
                    groups_coming.push(group);
                }
            }
        }
    }

    Some(Template::render(
        "post_overview",
        context! {
            post: post,
            groups_at_post: groups_at_post,
            groups_left: groups_left,
            groups_coming: groups_coming,
            is_admin: is_admin(cookies),
        },
    ))
}

pub fn routes() -> Vec<Route> {
    routes![post_overview]
}
