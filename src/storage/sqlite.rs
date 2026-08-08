//! SQLite persistence via rusqlite (bundled, single-file, no server).

use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection};

use super::models::{Edge, EdgeType, FileRecord, FileType, Node};

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS files (
    id          INTEGER PRIMARY KEY AUTOINCREMENT,
    path        TEXT NOT NULL UNIQUE,
    file_type   TEXT NOT NULL,
    title       TEXT NOT NULL,
    imported_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE TABLE IF NOT EXISTS nodes (
    id             INTEGER PRIMARY KEY AUTOINCREMENT,
    title          TEXT NOT NULL,
    content        TEXT NOT NULL,
    tags           TEXT NOT NULL DEFAULT '',
    source_file_id INTEGER NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    pos_x          REAL,
    pos_y          REAL
);
CREATE TABLE IF NOT EXISTS edges (
    id        INTEGER PRIMARY KEY AUTOINCREMENT,
    from_id   INTEGER NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    to_id     INTEGER NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    edge_type TEXT NOT NULL,
    weight    REAL NOT NULL DEFAULT 1.0
);
CREATE INDEX IF NOT EXISTS idx_nodes_file ON nodes(source_file_id);
CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_id);
CREATE INDEX IF NOT EXISTS idx_edges_to   ON edges(to_id);
";

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "foreign_keys", "ON")?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    // ---- files ----------------------------------------------------------

    pub fn file_id_for_path(&self, path: &str) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare("SELECT id FROM files WHERE path = ?1")?;
        let mut rows = stmt.query(params![path])?;
        Ok(match rows.next()? {
            Some(row) => Some(row.get(0)?),
            None => None,
        })
    }

    pub fn insert_file(&self, path: &str, file_type: &FileType, title: &str) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO files (path, file_type, title) VALUES (?1, ?2, ?3)",
            params![path, file_type.to_db_string(), title],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Removes a file record; its nodes (and their edges) cascade-delete.
    /// Never touches the file on disk.
    pub fn delete_file(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM files WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn load_files(&self) -> Result<Vec<FileRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, path, file_type, title, imported_at FROM files ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(FileRecord {
                id: row.get(0)?,
                path: row.get(1)?,
                file_type: FileType::from_db_string(&row.get::<_, String>(2)?),
                title: row.get(3)?,
                imported_at: row.get(4)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    // ---- nodes ----------------------------------------------------------

    /// Inserts a node (the `id` field of the argument is ignored) and returns the new id.
    pub fn insert_node(&self, node: &Node) -> Result<i64> {
        let (x, y) = match node.position {
            Some((x, y)) => (Some(x as f64), Some(y as f64)),
            None => (None, None),
        };
        self.conn.execute(
            "INSERT INTO nodes (title, content, tags, source_file_id, pos_x, pos_y)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                node.title,
                node.content,
                join_tags(&node.tags),
                node.source_file_id,
                x,
                y
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn update_node(&self, node: &Node) -> Result<()> {
        self.conn.execute(
            "UPDATE nodes SET title = ?2, content = ?3, tags = ?4 WHERE id = ?1",
            params![node.id, node.title, node.content, join_tags(&node.tags)],
        )?;
        Ok(())
    }

    /// Removes a node; its edges cascade-delete.
    pub fn delete_node(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM nodes WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn save_positions(&mut self, positions: &[(i64, f32, f32)]) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut stmt = tx.prepare("UPDATE nodes SET pos_x = ?2, pos_y = ?3 WHERE id = ?1")?;
            for (id, x, y) in positions {
                stmt.execute(params![id, *x as f64, *y as f64])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    pub fn load_nodes(&self) -> Result<Vec<Node>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, title, content, tags, source_file_id, pos_x, pos_y FROM nodes ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            let x: Option<f64> = row.get(5)?;
            let y: Option<f64> = row.get(6)?;
            Ok(Node {
                id: row.get(0)?,
                title: row.get(1)?,
                content: row.get(2)?,
                tags: split_tags(&row.get::<_, String>(3)?),
                source_file_id: row.get(4)?,
                position: x.zip(y).map(|(x, y)| (x as f32, y as f32)),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    // ---- edges ----------------------------------------------------------

    /// Inserts an edge (the `id` field of the argument is ignored) and returns the new id.
    pub fn insert_edge(&self, edge: &Edge) -> Result<i64> {
        self.conn.execute(
            "INSERT INTO edges (from_id, to_id, edge_type, weight) VALUES (?1, ?2, ?3, ?4)",
            params![
                edge.from_id,
                edge.to_id,
                edge.edge_type.as_str(),
                edge.weight as f64
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn load_edges(&self) -> Result<Vec<Edge>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, from_id, to_id, edge_type, weight FROM edges ORDER BY id")?;
        let rows = stmt.query_map([], |row| {
            Ok(Edge {
                id: row.get(0)?,
                from_id: row.get(1)?,
                to_id: row.get(2)?,
                edge_type: EdgeType::parse(&row.get::<_, String>(3)?),
                weight: row.get::<_, f64>(4)? as f32,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }
}

fn join_tags(tags: &[String]) -> String {
    tags.join(",")
}

fn split_tags(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(str::to_string)
        .collect()
}
