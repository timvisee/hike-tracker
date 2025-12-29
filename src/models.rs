use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub name: String,
    pub start_time: Option<DateTime<Utc>>,
    pub finish_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Post {
    pub id: String,
    pub name: String,
    pub order: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scan {
    pub id: String,
    pub group_id: String,
    pub post_id: String,
    pub arrival_time: DateTime<Utc>,
    pub departure_time: Option<DateTime<Utc>>,
}

impl Group {
    pub fn new(name: String) -> Self {
        Group {
            id: Uuid::new_v4().to_string(),
            name,
            start_time: None,
            finish_time: None,
            created_at: Utc::now(),
        }
    }

    pub fn insert(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO groups (id, name, start_time, finish_time, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.id,
                self.name,
                self.start_time.map(|t| t.to_rfc3339()),
                self.finish_time.map(|t| t.to_rfc3339()),
                self.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Group>> {
        let mut stmt = conn.prepare("SELECT id, name, start_time, finish_time, created_at FROM groups ORDER BY created_at DESC")?;
        let groups = stmt.query_map([], |row| {
            let start_time: Option<String> = row.get(2)?;
            let finish_time: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            Ok(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                start_time: start_time.map(|t| {
                    DateTime::parse_from_rfc3339(&t)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                finish_time: finish_time.map(|t| {
                    DateTime::parse_from_rfc3339(&t)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;
        groups.collect()
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Group>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, start_time, finish_time, created_at FROM groups WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let start_time: Option<String> = row.get(2)?;
            let finish_time: Option<String> = row.get(3)?;
            let created_at: String = row.get(4)?;
            Ok(Some(Group {
                id: row.get(0)?,
                name: row.get(1)?,
                start_time: start_time.map(|t| {
                    DateTime::parse_from_rfc3339(&t)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                finish_time: finish_time.map(|t| {
                    DateTime::parse_from_rfc3339(&t)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM scans WHERE group_id = ?1", params![id])?;
        conn.execute("DELETE FROM groups WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn set_start_time(conn: &Connection, id: &str, start_time: DateTime<Utc>) -> Result<()> {
        conn.execute(
            "UPDATE groups SET start_time = ?1 WHERE id = ?2",
            params![start_time.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn set_finish_time(conn: &Connection, id: &str, finish_time: DateTime<Utc>) -> Result<()> {
        conn.execute(
            "UPDATE groups SET finish_time = ?1 WHERE id = ?2",
            params![finish_time.to_rfc3339(), id],
        )?;
        Ok(())
    }
}

impl Post {
    pub fn new(name: String, order: i32) -> Self {
        Post {
            id: Uuid::new_v4().to_string(),
            name,
            order,
            created_at: Utc::now(),
        }
    }

    pub fn insert(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO posts (id, name, post_order, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![self.id, self.name, self.order, self.created_at.to_rfc3339(),],
        )?;
        Ok(())
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<Post>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, post_order, created_at FROM posts ORDER BY post_order ASC",
        )?;
        let posts = stmt.query_map([], |row| {
            let created_at: String = row.get(3)?;
            Ok(Post {
                id: row.get(0)?,
                name: row.get(1)?,
                order: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;
        posts.collect()
    }

    pub fn get_by_id(conn: &Connection, id: &str) -> Result<Option<Post>> {
        let mut stmt =
            conn.prepare("SELECT id, name, post_order, created_at FROM posts WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let created_at: String = row.get(3)?;
            Ok(Some(Post {
                id: row.get(0)?,
                name: row.get(1)?,
                order: row.get(2)?,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn delete(conn: &Connection, id: &str) -> Result<()> {
        conn.execute("DELETE FROM scans WHERE post_id = ?1", params![id])?;
        conn.execute("DELETE FROM posts WHERE id = ?1", params![id])?;
        Ok(())
    }
}

impl Scan {
    pub fn new(group_id: String, post_id: String) -> Self {
        Scan {
            id: Uuid::new_v4().to_string(),
            group_id,
            post_id,
            arrival_time: Utc::now(),
            departure_time: None,
        }
    }

    pub fn insert(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO scans (id, group_id, post_id, arrival_time, departure_time) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                self.id,
                self.group_id,
                self.post_id,
                self.arrival_time.to_rfc3339(),
                self.departure_time.map(|t| t.to_rfc3339()),
            ],
        )?;
        Ok(())
    }

    pub fn get_by_group_and_post(
        conn: &Connection,
        group_id: &str,
        post_id: &str,
    ) -> Result<Option<Scan>> {
        let mut stmt = conn.prepare(
            "SELECT id, group_id, post_id, arrival_time, departure_time FROM scans WHERE group_id = ?1 AND post_id = ?2",
        )?;
        let mut rows = stmt.query(params![group_id, post_id])?;
        if let Some(row) = rows.next()? {
            let arrival_time: String = row.get(3)?;
            let departure_time: Option<String> = row.get(4)?;
            Ok(Some(Scan {
                id: row.get(0)?,
                group_id: row.get(1)?,
                post_id: row.get(2)?,
                arrival_time: DateTime::parse_from_rfc3339(&arrival_time)
                    .unwrap()
                    .with_timezone(&Utc),
                departure_time: departure_time.map(|t| {
                    DateTime::parse_from_rfc3339(&t)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
            }))
        } else {
            Ok(None)
        }
    }

    pub fn set_departure_time(
        conn: &Connection,
        id: &str,
        departure_time: DateTime<Utc>,
    ) -> Result<()> {
        conn.execute(
            "UPDATE scans SET departure_time = ?1 WHERE id = ?2",
            params![departure_time.to_rfc3339(), id],
        )?;
        Ok(())
    }

    pub fn get_by_group(conn: &Connection, group_id: &str) -> Result<Vec<Scan>> {
        let mut stmt = conn.prepare(
            "SELECT s.id, s.group_id, s.post_id, s.arrival_time, s.departure_time
             FROM scans s
             JOIN posts p ON s.post_id = p.id
             WHERE s.group_id = ?1
             ORDER BY p.post_order ASC",
        )?;
        let scans = stmt.query_map(params![group_id], |row| {
            let arrival_time: String = row.get(3)?;
            let departure_time: Option<String> = row.get(4)?;
            Ok(Scan {
                id: row.get(0)?,
                group_id: row.get(1)?,
                post_id: row.get(2)?,
                arrival_time: DateTime::parse_from_rfc3339(&arrival_time)
                    .unwrap()
                    .with_timezone(&Utc),
                departure_time: departure_time.map(|t| {
                    DateTime::parse_from_rfc3339(&t)
                        .unwrap()
                        .with_timezone(&Utc)
                }),
            })
        })?;
        scans.collect()
    }
}
