use chrono::{NaiveDateTime, TimeDelta, Utc};
use rocket::http::CookieJar;
use rocket::Route;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::auth::{self, AuthSession};
use crate::db::DbConn;
use crate::models::{Group, Post, Scan};

#[derive(Serialize)]
pub struct PostScanInfo {
    pub post: Post,
    pub scan: Option<Scan>,
    pub idle_time: Option<TimeDelta>,
}

#[derive(Serialize)]
pub struct GroupDetail {
    pub group: Group,
    pub post_scans: Vec<PostScanInfo>,
    pub total_time: Option<TimeDelta>,
    pub idle_time: TimeDelta,
    pub walking_time: Option<TimeDelta>,
}

#[get("/")]
pub async fn dashboard(cookies: &CookieJar<'_>, conn: DbConn) -> Template {
    let is_admin = auth::is_admin(cookies);
    let current_auth = auth::get_current_auth(cookies);
    let holder_post_id = match &current_auth {
        Some(AuthSession::PostHolder { post_id }) => Some(post_id.clone()),
        _ => None,
    };
    let is_post_holder = holder_post_id.is_some();

    let groups = conn.run(Group::get_all).await.unwrap_or_default();
    let posts = conn.run(Post::get_all).await.unwrap_or_default();

    let mut group_stats: Vec<GroupDetail> = Vec::new();

    for group in groups {
        let gid = group.id.clone();
        let scans = conn
            .run(move |c| Scan::get_by_group(c, &gid))
            .await
            .unwrap_or_default();
        group_stats.push(group_detail(group, &scans, posts.clone()));
    }

    Template::render(
        "dashboard",
        context! {
            group_stats: group_stats,
            posts: posts,
            is_admin: is_admin,
            is_post_holder: is_post_holder,
            holder_post_id: holder_post_id,
        },
    )
}

fn now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}

fn group_detail(group: Group, scans: &[Scan], posts: Vec<Post>) -> GroupDetail {
    let post_scans: Vec<PostScanInfo> = posts
        .into_iter()
        .map(|post| {
            let scan = scans.iter().find(|s| s.post_id == post.id).cloned();
            let idle_time = scan
                .as_ref()
                .map(|scan| scan.departure_time.unwrap_or_else(now_naive) - scan.arrival_time);
            PostScanInfo {
                post,
                scan,
                idle_time,
            }
        })
        .collect();

    let idle_time = post_scans.iter().filter_map(|ps| ps.idle_time).sum();

    let total_time = group
        .start_time
        .map(|start| group.finish_time.unwrap_or_else(now_naive) - start);

    let walking_time = total_time.map(|t| t - idle_time);

    GroupDetail {
        group,
        post_scans,
        total_time,
        idle_time,
        walking_time,
    }
}

pub fn routes() -> Vec<Route> {
    routes![dashboard]
}
