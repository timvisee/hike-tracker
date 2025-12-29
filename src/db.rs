use rusqlite::{Connection, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.run_migrations()?;
        Ok(db)
    }

    fn run_migrations(&self) -> Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS groups (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                start_time TEXT,
                finish_time TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS posts (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                post_order INTEGER NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS scans (
                id TEXT PRIMARY KEY,
                group_id TEXT NOT NULL,
                post_id TEXT NOT NULL,
                arrival_time TEXT NOT NULL,
                departure_time TEXT,
                FOREIGN KEY (group_id) REFERENCES groups(id),
                FOREIGN KEY (post_id) REFERENCES posts(id)
            );
            ",
        )?;
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
