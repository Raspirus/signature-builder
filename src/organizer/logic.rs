use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use log::{info, warn};

use super::database::{create_pool, insert_hashes, remove_hashes};

pub fn patch(database: String, table_name: String, file_name: String) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let file = File::open(file_name)?;
    let bufreader = BufReader::new(file);

    let mut add = Vec::new();
    let mut remove = Vec::new();

    for line in bufreader.lines() {
        let line = line?;
        match line {
            _ if line.starts_with("+") => add.push(line.replacen("+", "", 1)),
            _ if line.starts_with("-") => remove.push(line.replacen("-", "", 1)),
            _ => warn!("Ignoring line {line}"),
        }
    }

    let mut database = create_pool(database, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    info!("Adding {} hashes from patch...", add.len());
    insert_hashes(&mut database, table_name.clone(), &add)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    info!("Removing {} hashes from patch...", remove.len());
    remove_hashes(&mut database, table_name, &remove)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    info!(
        "Patching took {}s",
        std::time::Instant::now()
            .duration_since(start_time)
            .as_secs_f32()
    );
    Ok(())
}
