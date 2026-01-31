use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::auth::{get_auth_context, is_admin};
use crate::db::DbConn;
use crate::models::{Group, Post, Scan};

#[derive(Serialize)]
struct GroupStatus {
    group: Group,
    scan: Option<Scan>,
    arrival_time: Option<String>,
    departure_time: Option<String>,
    time_at_post: Option<String>,
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
    let all_groups = conn.run(Group::get_all).await.unwrap_or_default();

    let mut groups_at_post = Vec::new();
    let mut groups_left = Vec::new();
    let mut groups_coming = Vec::new();

    for group in all_groups {
        // Check if this group has a scan at this post
        let scan = scans.iter().find(|s| s.group_id == group.id).cloned();

        match &scan {
            Some(s) => {
                let arrival_time = Some(s.arrival_time.format("%H:%M:%S").to_string());
                let departure_time = s.departure_time.map(|dt| dt.format("%H:%M:%S").to_string());
                let time_at_post = s.departure_time.map(|dt| {
                    let duration = dt - s.arrival_time;
                    let total_secs = duration.num_seconds();
                    let hours = total_secs / 3600;
                    let minutes = (total_secs % 3600) / 60;
                    let seconds = total_secs % 60;
                    format!("{hours:02}:{minutes:02}:{seconds:02}")
                });

                let status = GroupStatus {
                    group: group.clone(),
                    scan: scan.clone(),
                    arrival_time,
                    departure_time: departure_time.clone(),
                    time_at_post,
                };

                if s.departure_time.is_some() {
                    groups_left.push(status);
                } else {
                    groups_at_post.push(status);
                }
            }
            None => {
                // Group hasn't arrived at this post yet - show as coming
                groups_coming.push(GroupStatus {
                    group,
                    scan: None,
                    arrival_time: None,
                    departure_time: None,
                    time_at_post: None,
                });
            }
        }
    }

    // Check if current user is the post holder for this post
    let auth_ctx = get_auth_context(cookies);
    let is_post_holder = matches!(
        auth_ctx.holder_post_id.as_ref(),
        Some(pid) if pid == &post_id
    );
    let holder_post_id = auth_ctx.holder_post_id;

    Some(Template::render(
        "post_overview",
        context! {
            post: post,
            groups_at_post: groups_at_post,
            groups_left: groups_left,
            groups_coming: groups_coming,
            is_admin: is_admin(cookies),
            is_post_holder: is_post_holder,
            holder_post_id: holder_post_id,
        },
    ))
}

pub fn routes() -> Vec<Route> {
    routes![post_overview]
}
