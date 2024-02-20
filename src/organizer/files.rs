use std::{
    fs::{self, DirEntry, File},
    io::{BufRead, BufReader, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use log::{debug, error, info, warn};

use crate::organizer::database::{
    create_pool, get_hash_count, get_hashes, insert_hashes, remove_hashes,
};

/// inserts the content of provided file into database
pub fn insert_file(file_path: String, database: String, table_name: String) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();

    let mut database = create_pool(database, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let mut lines: Vec<String> = Vec::new();
    let file = File::open(&file_path)?;
    let reader = BufReader::new(file);
    // reads line by line from file
    for line in reader.lines() {
        // if line starts with # we ignore it, otherwise we push
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

    // insert into database
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

/// inserts the content of files in provided folder into database
pub fn insert_files(
    tmp_dir: String,
    max_file_combines: usize,
    database: String,
    table_name: String,
) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();

    // get all files from a folder
    let entries: Vec<DirEntry> = fs::read_dir(Path::new(&tmp_dir))?
        .filter_map(Result::ok)
        .collect();
    let output_dir = Path::new(&tmp_dir);

    let mut database = create_pool(database, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    // combine files
    for chunk_id in 0..=(entries.len() / max_file_combines) {
        // compute which frame of files to read
        let start = chunk_id * max_file_combines;
        let end = std::cmp::min((chunk_id + 1) * max_file_combines, entries.len());

        // read all files line by line into buffer
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
                // if line starts with # we ignore otherwise we push
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
        // insert into databse
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

/// patches the database with the supplied file
pub fn patch(database: String, table_name: String, file_name: String) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let file = File::open(file_name)?;
    let bufreader = BufReader::new(file);

    let mut add = Vec::new();
    let mut remove = Vec::new();

    // we read over each line
    for line in bufreader.lines() {
        let line = line?;
        match line {
            _ if line.starts_with('+') => add.push(line.replacen('+', "", 1).trim().to_owned()),
            _ if line.starts_with('-') => remove.push(line.replacen('-', "", 1).trim().to_owned()),
            _ => warn!("Ignoring line {line}"),
        }
    }

    // insert into database
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

/// writes the database hashes to output files
pub fn write_files(
    output_dir_string: String,
    file_size: usize,
    database: String,
    table_name: String,
) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    let output_dir = Path::new(&output_dir_string);
    // removes director if exist
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)?;
    }
    fs::create_dir_all(output_dir)?;

    // setup connection
    let connection = create_pool(database, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;

    let count = get_hash_count(&connection, table_name.clone())
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))?;
    info!("Exporting {count} hashes...");

    let mut current_frame = 0;
    let mut current_file = 0;
    loop {
        // create frames
        let bottom = current_frame * file_size;
        let top = bottom + file_size;
        // fetch hashes for current frame from
        let hashes = get_hashes(&connection, table_name.clone(), bottom, top).map_err(|err| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to fetch hashes from database: {err}"),
            )
        })?;

        // if no more hashes have been found, we are done
        if hashes.is_empty() {
            break;
        }
        // determining output filename
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

/// writes a timestamp file to the output repository
pub fn set_timestamp(output_dir: String) -> std::io::Result<()> {
    let current_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?
        .as_millis();
    let timestamp = Path::new(&output_dir).join("timestamp");
    if timestamp.exists() {
        fs::remove_file(&timestamp)?;
    }
    let mut file = File::create(timestamp)?;
    file.write_all(format!("{current_timestamp}").as_bytes())
}
