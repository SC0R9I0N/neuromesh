//! Dev helper: dump the contents of the user's NeuroMesh database.
//! Usage: cargo run --example dbdump

fn main() {
    let path = std::path::PathBuf::from(std::env::var("APPDATA").expect("APPDATA not set"))
        .join("NeuroMesh")
        .join("neuromesh.db");
    println!("db: {}", path.display());
    let conn = rusqlite::Connection::open(&path).expect("open db");

    for table in ["files", "nodes", "edges"] {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            .unwrap_or(-1);
        println!("{table}: {count}");
    }

    let mut stmt = conn
        .prepare("SELECT id, title, file_type, path FROM files ORDER BY id")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        println!(
            "file #{}: {} [{}] {}",
            row.get::<_, i64>(0).unwrap(),
            row.get::<_, String>(1).unwrap(),
            row.get::<_, String>(2).unwrap(),
            row.get::<_, String>(3).unwrap()
        );
    }

    let mut stmt = conn
        .prepare("SELECT id, title, tags FROM nodes ORDER BY id LIMIT 30")
        .unwrap();
    let mut rows = stmt.query([]).unwrap();
    while let Some(row) = rows.next().unwrap() {
        println!(
            "node #{}: {} [{}]",
            row.get::<_, i64>(0).unwrap(),
            row.get::<_, String>(1).unwrap(),
            row.get::<_, String>(2).unwrap()
        );
    }
}
