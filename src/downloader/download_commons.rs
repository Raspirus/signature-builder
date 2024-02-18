use std::{
    fs::{self, DirEntry, File},
    io::{BufRead, BufReader, Write},
    path::Path,
};

use log::{debug, error, info, warn};
use reqwest::StatusCode;

use crate::organizer::database::{create_pool, create_table, get_hashes, insert_hashes};

/// downloads a file from file_url and save it to output_name. it expects the path to the output name to already exist
pub fn download_file(
    output_name: &Path,
    file_url: &str,
    max_retries: usize,
) -> std::io::Result<()> {
    output_name.exists().then(|| fs::remove_file(output_name));
    let mut file = File::create(output_name)?;
    let client = reqwest::blocking::Client::new();

    for current_retry in 0..=max_retries {
        let response = match client.get(file_url).send() {
            Ok(response) => response,
            Err(err) => {
                warn!("Failed to download {file_url} on try {current_retry}: {err}");
                continue;
            }
        };

        match response.status() {
            StatusCode::OK => match response.text() {
                Ok(data) => file.write_all(data.as_bytes())?,
                Err(err) => warn!("Failed to download {file_url} on try {current_retry}: {err}"),
            },
            _ => warn!(
                "Failed to download {file_url} on try {current_retry}; Statuscode was {}",
                response.status()
            ),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::ConnectionAborted,
        "Could not download file",
    ))
}

/// writes the database hashes to output files
pub fn write_files(
    output_dir_string: String,
    file_size: usize,
    database: String,
    table_name: String,
) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let output_dir = Path::new(&output_dir_string);
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    let files: Vec<DirEntry> = fs::read_dir(output_dir)?.filter_map(Result::ok).collect();
    let mut max = 0;
    for file in files {
        let out = file
            .file_name()
            .to_str()
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or_default();
        if out > max {
            max = out
        }
    }
    if max > 0 {
        max += 1
    }

    let connection = create_pool(database).expect("Failed to get connection");
    let mut current_frame = 0;
    let mut current_file = max;
    loop {
        let bottom = current_frame * file_size;
        let top = bottom + file_size;
        let hashes = get_hashes(&connection, table_name.clone(), bottom, top)
            .expect("Failed to fetch hashes from db");
        if hashes.is_empty() {
            break;
        }
        let mut file = File::create(Path::new(&format!(
            "{output_dir_string}/{:0>5}",
            current_file
        )))?;
        info!("Writing to {output_dir_string}/{:0>5}", current_file);
        for hash in &hashes {
            writeln!(file, "{}", hash)?;
        }
        current_file += 1;
        current_frame += 1;
    }
    info!(
        "Writing output files took {}s",
        std::time::Instant::now()
            .duration_since(start_time)
            .as_secs()
    );
    Ok(())
}

pub fn insert_file(file_path: String, database: String, table_name: String) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();

    let mut database = create_pool(database)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    create_table(&database, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let mut lines: Vec<String> = Vec::new();
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    for line in reader.lines() {
        match line {
            Ok(line) => {
                if !line.starts_with('#') {
                    lines.push(line)
                }
            }
            Err(err) => {
                warn!("Could not read line in file {}: {err}", file_path);
                continue;
            }
        };
    }

    info!(
        "Inserting file {} containing {} hashes into database...",
        file_path,
        lines.len()
    );

    insert_hashes(&mut database, table_name.clone(), &lines)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    info!(
        "Inserted file in {}s",
        std::time::Instant::now()
            .duration_since(start_time)
            .as_secs_f32()
    );
    Ok(())
}

pub fn insert_files(
    tmp_dir: String,
    max_file_combines: usize,
    database: String,
    table_name: String,
) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(&tmp_dir))?
        .filter_map(Result::ok)
        .collect();
    let output_dir = Path::new(&tmp_dir);

    let mut database = create_pool(database)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    create_table(&database, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    for chunk_id in 0..=(entries.len() / max_file_combines) {
        let start = chunk_id * max_file_combines;
        let end = std::cmp::min((chunk_id + 1) * max_file_combines, entries.len());

        let mut lines: Vec<String> = Vec::new();
        for file_id in start..end {
            let reader_path = output_dir.join(
                entries
                    .get(file_id)
                    .ok_or_else(|| {
                        std::io::Error::new(std::io::ErrorKind::Other, "Could not get DirEntry")
                    })?
                    .file_name(),
            );
            debug!("Adding {} to batch", reader_path.display());
            let file = match File::open(&reader_path) {
                Ok(file) => file,
                Err(err) => {
                    error!(
                        "Could not open file {} for reading: {err}",
                        reader_path.display()
                    );
                    continue;
                }
            };
            let reader = BufReader::new(file);
            for line in reader.lines() {
                match line {
                    Ok(line) => {
                        if !line.starts_with('#') {
                            lines.push(line)
                        }
                    }
                    Err(err) => {
                        warn!(
                            "Could not read line in file {}: {err}",
                            reader_path.display()
                        );
                        continue;
                    }
                };
            }
        }

        info!(
            "Inserting chunk {}/{} containing {} hashes into database...",
            chunk_id,
            (entries.len() / max_file_combines),
            lines.len()
        );
        match insert_hashes(&mut database, table_name.clone(), &lines) {
            Ok(_) => {}
            Err(err) => {
                warn!("Error inserting: {err}");
            }
        }
    }
    info!(
        "Building database took {}s",
        std::time::Instant::now()
            .duration_since(start_time)
            .as_secs_f32()
    );
    Ok(())
}

pub fn cleanup(tmp_dir: String, database: String) {
    info!("Deleting temp folder...");
    fs::remove_dir_all(tmp_dir).unwrap_or(warn!("Temporary directory does not exist; Skipping..."));
    info!("Deleting database...");
    fs::remove_file(database).unwrap_or(warn!("Database file does not exist; Skipping..."));
}
