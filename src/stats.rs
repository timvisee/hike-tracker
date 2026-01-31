use chrono::{NaiveDateTime, TimeDelta, Utc};
use serde::Serialize;

use crate::models::{Group, Post, Scan};

#[derive(Serialize, Clone)]
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

pub fn now_naive() -> NaiveDateTime {
    Utc::now().naive_utc()
}

pub fn calculate_group_stats(group: &Group, scans: &[Scan], posts: Vec<Post>) -> GroupStats {
    let post_scans: Vec<PostScanInfo> = posts
        .into_iter()
        .map(|post| {
            let scan = scans.iter().find(|s| s.post_id == post.id).cloned();
            let idle_time = scan
                .as_ref()
                .map(|s| s.departure_time.unwrap_or_else(now_naive) - s.arrival_time);
            PostScanInfo {
                post,
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

pub fn format_duration(delta: TimeDelta) -> String {
    let total_secs = delta.num_seconds();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}
