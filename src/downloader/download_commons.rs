use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

use log::warn;
use reqwest::StatusCode;

/// downloads a file from file_url and save it to output_name. output folder needs to exist or function will throw error
pub fn download_file(
    output_name: &Path,
    file_url: &str,
    max_retries: usize,
) -> std::io::Result<()> {
    // checks if output folder exists
    match output_name.parent() {
        Some(parent_dir) => if !parent_dir.exists() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Parent directory does not exist",
            ))
        } else {
            Ok(())
        }
        None => Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "No parent directory",
        )),
    }?;
    // deletes output file if exist
    output_name.exists().then(|| fs::remove_file(output_name));

    let mut file = File::create(output_name)?;
    let client = reqwest::blocking::Client::new();

    // retry until max_retries is reached or download succeeded
    for current_retry in 0..=max_retries {
        let response = match client.get(file_url).send() {
            Ok(response) => response,
            Err(err) => {
                warn!("Failed to download {file_url} on try {current_retry}: {err}");
                continue;
            }
        };
        // if ok we write to file, otherwise we retry
        match response.status() {
            StatusCode::OK => match response.text() {
                Ok(data) => {
                    file.write_all(data.as_bytes())?;
                    return Ok(())
                },
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
