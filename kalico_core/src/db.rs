use duckdb::{Connection, Result as DuckResult};
use std::sync::Mutex;

pub struct MeshDB {
    conn: Mutex<Connection>,
}

impl MeshDB {
    pub fn new(path: &str) -> DuckResult<Self> {
        let conn = Connection::open(path)?;
        // Create tables if not exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS mesh_points (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                x REAL, y REAL, z REAL, kind TEXT, ts TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );"
        )?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn add_mesh(&self, points: &[(f32, f32, f32)], kind: &str) -> anyhow::Result<()> {
        let conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        for &(x, y, z) in points {
            tx.execute(
                "INSERT INTO mesh_points (x, y, z, kind) VALUES (?, ?, ?, ?)",
                (x, y, z, kind),
            )?;
        }
        tx.commit()?;
        Ok(())
    }
}
