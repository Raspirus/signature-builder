use std::{fs, path::PathBuf, sync::Arc};

use super::download_commons::download_file;
use log::{error, info, trace, warn};
use reqwest::StatusCode;
use threadpool_rs::threadpool::pool::ThreadPool;

static URL: &str = "https://virusshare.com/hashfiles/VirusShare_";

/// downloads all files from provider into output_dir (tmp workfolder)
pub fn download_all(
    output_dir: Arc<PathBuf>,
    max_threads: usize,
    max_retries: usize,
) -> std::io::Result<()> {
    let start_time = std::time::Instant::now();
    // creates output folder
    fs::create_dir_all(output_dir.as_ref())?;

    info!("Indexing webfiles...");
    let filecount = match get_file_count(max_retries) {
        Ok(filecount) => filecount,
        Err(err) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Could not get maximum filecount: {err}"),
            ))
        }
    };
    info!("Found {filecount} file(s)");
    // multithreaded download
    let pool = ThreadPool::new(max_threads)?;
    for file_id in 0..=filecount {
        let dir = output_dir.clone();
        pool.execute(move || {
            let download_path = dir.join(format!("vs_{:0>5}.md5", file_id));
            let file_url = format!("{URL}{:0>5}.md5", file_id);
            match download_file(&download_path, &file_url, max_retries) {
                Ok(_) => info!("Downloaded {}", download_path.display()),
                Err(err) => error!("Failed to download {file_url}: {err}"),
            };
        });
    }
    // wait for files to finish downloading
    drop(pool);

    info!(
        "Downloaded files in {}s",
        std::time::Instant::now()
            .duration_since(start_time)
            .as_secs()
    );
    Ok(())
}

/// calculates the total number of files present on provider
fn get_file_count(base_max_retry: usize) -> Result<usize, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let mut max = 0;
    let mut max_retry = base_max_retry;

    // go up in 10 increments
    loop {
        let file_url = format!("{URL}{:0>5}.md5", max);
        trace!("Requesting {}", file_url);
        let response = client.head(file_url).send()?;
        match response.status() {
            StatusCode::OK => max += 10,
            StatusCode::NOT_FOUND => break,
            _ => {
                warn!(
                    "Received invalid status {}, trying again...",
                    response.status()
                );
                max_retry -= 1;
                if max_retry == 0 {
                    warn!("Failed 5 times, aborting; Check your network?")
                }
            }
        }
    }

    max -= 10;
    max_retry = base_max_retry;

    // go up in 1 increments from last 10th still present
    loop {
        let file_url = format!("{URL}{:0>5}.md5", max);
        trace!("Requesting {}", file_url);
        let response = client.head(file_url).send()?;
        match response.status() {
            StatusCode::OK => max += 1,
            StatusCode::NOT_FOUND => break,
            _ => {
                warn!(
                    "Received invalid status {}, trying again...",
                    response.status()
                );
                max_retry -= 1;
                if max_retry == 0 {
                    warn!("Failed 5 times, aborting; Check your network?")
                }
            }
        }
    }
    Ok(max - 1)
}
