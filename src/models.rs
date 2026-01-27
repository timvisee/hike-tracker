use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::{groups, posts, scans};

// ============ GROUP MODELS ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = groups)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Group {
    pub id: String,
    pub name: String,
    pub scout_group: String,
    pub members: String,
    pub phone_number: String,
    pub start_time: Option<NaiveDateTime>,
    pub finish_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = groups)]
pub struct NewGroup {
    pub id: String,
    pub name: String,
    pub scout_group: String,
    pub members: String,
    pub phone_number: String,
    pub start_time: Option<NaiveDateTime>,
    pub finish_time: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

impl NewGroup {
    pub fn new(name: String, scout_group: String, members: String, phone_number: String) -> Self {
        NewGroup {
            id: Uuid::new_v4().to_string(),
            name,
            scout_group,
            members,
            phone_number,
            start_time: None,
            finish_time: None,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }

    pub fn new_with_id(
        id: String,
        name: String,
        scout_group: String,
        members: String,
        phone_number: String,
    ) -> Self {
        NewGroup {
            id,
            name,
            scout_group,
            members,
            phone_number,
            start_time: None,
            finish_time: None,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl Group {
    pub fn insert(conn: &mut SqliteConnection, new_group: NewGroup) -> QueryResult<usize> {
        diesel::insert_into(groups::table)
            .values(&new_group)
            .execute(conn)
    }

    pub fn get_all(conn: &mut SqliteConnection) -> QueryResult<Vec<Group>> {
        groups::table
            .order(groups::created_at.desc())
            .load::<Group>(conn)
    }

    pub fn get_by_id(conn: &mut SqliteConnection, group_id: &str) -> QueryResult<Option<Group>> {
        groups::table
            .filter(groups::id.eq(group_id))
            .first::<Group>(conn)
            .optional()
    }

    pub fn delete(conn: &mut SqliteConnection, group_id: &str) -> QueryResult<usize> {
        // Delete associated scans first (cascade)
        diesel::delete(scans::table.filter(scans::group_id.eq(group_id))).execute(conn)?;
        // Delete the group
        diesel::delete(groups::table.filter(groups::id.eq(group_id))).execute(conn)
    }

    pub fn set_start_time(
        conn: &mut SqliteConnection,
        group_id: &str,
        start_time: NaiveDateTime,
    ) -> QueryResult<usize> {
        diesel::update(groups::table.filter(groups::id.eq(group_id)))
            .set(groups::start_time.eq(Some(start_time)))
            .execute(conn)
    }

    pub fn clear_start_time(conn: &mut SqliteConnection, group_id: &str) -> QueryResult<usize> {
        diesel::update(groups::table.filter(groups::id.eq(group_id)))
            .set(groups::start_time.eq(None::<NaiveDateTime>))
            .execute(conn)
    }

    pub fn set_finish_time(
        conn: &mut SqliteConnection,
        group_id: &str,
        finish_time: NaiveDateTime,
    ) -> QueryResult<usize> {
        diesel::update(groups::table.filter(groups::id.eq(group_id)))
            .set(groups::finish_time.eq(Some(finish_time)))
            .execute(conn)
    }

    pub fn clear_finish_time(conn: &mut SqliteConnection, group_id: &str) -> QueryResult<usize> {
        diesel::update(groups::table.filter(groups::id.eq(group_id)))
            .set(groups::finish_time.eq(None::<NaiveDateTime>))
            .execute(conn)
    }
}

// ============ POST MODELS ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = posts)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Post {
    pub id: String,
    pub name: String,
    pub post_order: i32,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = posts)]
pub struct NewPost {
    pub id: String,
    pub name: String,
    pub post_order: i32,
    pub created_at: NaiveDateTime,
}

impl NewPost {
    pub fn new(name: String, order: i32) -> Self {
        NewPost {
            id: Uuid::new_v4().to_string(),
            name,
            post_order: order,
            created_at: chrono::Utc::now().naive_utc(),
        }
    }
}

impl Post {
    pub fn insert(conn: &mut SqliteConnection, new_post: NewPost) -> QueryResult<usize> {
        diesel::insert_into(posts::table)
            .values(&new_post)
            .execute(conn)
    }

    pub fn get_all(conn: &mut SqliteConnection) -> QueryResult<Vec<Post>> {
        posts::table
            .order(posts::post_order.asc())
            .load::<Post>(conn)
    }

    pub fn get_by_id(conn: &mut SqliteConnection, post_id: &str) -> QueryResult<Option<Post>> {
        posts::table
            .filter(posts::id.eq(post_id))
            .first::<Post>(conn)
            .optional()
    }

    pub fn delete(conn: &mut SqliteConnection, post_id: &str) -> QueryResult<usize> {
        // Delete associated scans first (cascade)
        diesel::delete(scans::table.filter(scans::post_id.eq(post_id))).execute(conn)?;
        // Delete the post
        diesel::delete(posts::table.filter(posts::id.eq(post_id))).execute(conn)
    }
}

// ============ SCAN MODELS ============

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = scans)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Scan {
    pub id: String,
    pub group_id: String,
    pub post_id: String,
    pub arrival_time: NaiveDateTime,
    pub departure_time: Option<NaiveDateTime>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = scans)]
pub struct NewScan {
    pub id: String,
    pub group_id: String,
    pub post_id: String,
    pub arrival_time: NaiveDateTime,
    pub departure_time: Option<NaiveDateTime>,
}

impl NewScan {
    pub fn new(group_id: String, post_id: String) -> Self {
        NewScan {
            id: Uuid::new_v4().to_string(),
            group_id,
            post_id,
            arrival_time: chrono::Utc::now().naive_utc(),
            departure_time: None,
        }
    }
}

impl Scan {
    pub fn insert(conn: &mut SqliteConnection, new_scan: NewScan) -> QueryResult<usize> {
        diesel::insert_into(scans::table)
            .values(&new_scan)
            .execute(conn)
    }

    pub fn get_by_group_and_post(
        conn: &mut SqliteConnection,
        group_id: &str,
        post_id: &str,
    ) -> QueryResult<Option<Scan>> {
        scans::table
            .filter(scans::group_id.eq(group_id))
            .filter(scans::post_id.eq(post_id))
            .first::<Scan>(conn)
            .optional()
    }

    pub fn get_by_group(conn: &mut SqliteConnection, group_id: &str) -> QueryResult<Vec<Scan>> {
        // Join with posts to order by post_order
        scans::table
            .inner_join(posts::table)
            .filter(scans::group_id.eq(group_id))
            .order(posts::post_order.asc())
            .select(Scan::as_select())
            .load::<Scan>(conn)
    }

    pub fn set_departure_time(
        conn: &mut SqliteConnection,
        scan_id: &str,
        departure_time: NaiveDateTime,
    ) -> QueryResult<usize> {
        diesel::update(scans::table.filter(scans::id.eq(scan_id)))
            .set(scans::departure_time.eq(Some(departure_time)))
            .execute(conn)
    }

    pub fn clear_departure_time(conn: &mut SqliteConnection, scan_id: &str) -> QueryResult<usize> {
        diesel::update(scans::table.filter(scans::id.eq(scan_id)))
            .set(scans::departure_time.eq(None::<NaiveDateTime>))
            .execute(conn)
    }

    pub fn set_arrival_time(
        conn: &mut SqliteConnection,
        scan_id: &str,
        arrival_time: NaiveDateTime,
    ) -> QueryResult<usize> {
        diesel::update(scans::table.filter(scans::id.eq(scan_id)))
            .set(scans::arrival_time.eq(arrival_time))
            .execute(conn)
    }

    pub fn delete(conn: &mut SqliteConnection, scan_id: &str) -> QueryResult<usize> {
        diesel::delete(scans::table.filter(scans::id.eq(scan_id))).execute(conn)
    }

    pub fn get_by_post(conn: &mut SqliteConnection, post_id: &str) -> QueryResult<Vec<Scan>> {
        scans::table
            .filter(scans::post_id.eq(post_id))
            .load::<Scan>(conn)
    }
}
