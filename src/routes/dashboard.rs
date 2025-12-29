use chrono::TimeDelta;
use chrono::Utc;
use rocket::http::Status;
use rocket::Route;
use rocket::State;
use rocket_dyn_templates::{context, Template};
use serde::Serialize;

use crate::models::{Group, Post, Scan};
use crate::AppState;

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
pub fn dashboard(state: &State<AppState>) -> Template {
    let db = state.db.lock().unwrap();
    let groups = Group::get_all(db.conn()).unwrap_or_default();
    let posts = Post::get_all(db.conn()).unwrap_or_default();

    let group_stats: Vec<GroupDetail> = groups
        .iter()
        .map(|group| {
            let scans = Scan::get_by_group(db.conn(), &group.id).unwrap_or_default();
            group_detail(group.clone(), &scans, posts.clone())
        })
        .collect();

    Template::render(
        "dashboard",
        context! { group_stats: group_stats, total_posts: posts.len()},
    )
}

#[get("/group/<id>")]
pub fn group_detail_page(state: &State<AppState>, id: &str) -> Result<Template, Status> {
    let db = state.db.lock().unwrap();

    let group = match Group::get_by_id(db.conn(), id).ok().flatten() {
        Some(g) => g,
        None => return Err(Status::BadRequest),
    };
    let posts = Post::get_all(db.conn()).unwrap_or_default();
    let scans = Scan::get_by_group(db.conn(), group.id.as_ref()).unwrap_or_default();

    let detail = group_detail(group, &scans, posts);

    Ok(Template::render(
        "dashboard_detail",
        context! { detail: detail },
    ))
}

fn group_detail(group: Group, scans: &[Scan], posts: Vec<Post>) -> GroupDetail {
    let post_scans: Vec<PostScanInfo> = posts
        .into_iter()
        .map(|post| {
            let scan = scans.iter().find(|s| s.post_id == post.id).cloned();
            let idle_time = scan
                .as_ref()
                .map(|scan| scan.departure_time.unwrap_or_else(Utc::now) - scan.arrival_time);
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
        .map(|start| group.finish_time.unwrap_or_else(Utc::now) - start);

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
    routes![dashboard, group_detail_page]
}
