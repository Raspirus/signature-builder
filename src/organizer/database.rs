use log::trace;
use rusqlite::params;

pub fn create_table(
    connection: &rusqlite::Connection,
    table_name: String,
) -> Result<(), rusqlite::Error> {
    connection.execute(
        &format!(
            "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, hash TEXT NOT NULL UNIQUE)",
            table_name
        ),
        [],
    )?;
    Ok(())
}

pub fn insert_hashes(
    connection: &mut rusqlite::Connection,
    table_name: String,
    hashes: &Vec<String>,
) -> Result<(), rusqlite::Error> {
    let transaction = connection.transaction()?;
    for hash in hashes {
        trace!("Inserting {hash}");
        transaction.execute(
            &format!("INSERT OR IGNORE INTO {} (hash) VALUES (?1)", table_name),
            params![hash],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

pub fn remove_hashes(
    connection: &mut rusqlite::Connection,
    table_name: String,
    hashes: &Vec<String>,
) -> Result<(), rusqlite::Error> {
    let transaction = connection.transaction()?;
    for hash in hashes {
        trace!("Removing {hash}");
        transaction.execute(
            &format!("DELETE FROM {} WHERE hash = (?1)", table_name),
            params![hash],
        )?;
    }
    transaction.commit()?;
    Ok(())
}

pub fn get_hashes(
    connection: &rusqlite::Connection,
    table_name: String,
    bottom_index: usize,
    top_index: usize,
) -> Result<Vec<String>, rusqlite::Error> {
    let mut sql = connection.prepare(&format!(
        "SELECT hash FROM {} WHERE id >= ?1 AND id < ?2",
        table_name
    ))?;
    let hashes: Result<Vec<String>, rusqlite::Error> = sql
        .query_map(params![bottom_index, top_index], |row| row.get(0))?
        .collect();
    let out = hashes.unwrap_or_default();
    Ok(out)
}

pub fn get_hash_count(
    connection: &rusqlite::Connection,
    table_name: String,
) -> Result<u64, rusqlite::Error> {
    let mut sql = connection.prepare(&format!("SELECT COUNT(*) FROM {}", table_name))?;
    sql.query_row([], |row| row.get(0))
}

pub fn create_pool(
    database: String,
    table_name: String,
) -> Result<rusqlite::Connection, rusqlite::Error> {
    let connection = rusqlite::Connection::open(database)?;
    create_table(&connection, table_name.clone())?;
    Ok(connection)
}
