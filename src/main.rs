use std::{path::Path, process::exit, sync::Arc};

use cali::parser::Parser;
use log::{debug, error, info};

use crate::{
    downloader::virusshare::download_all,
    organizer::{
        database::{create_pool, get_hash_count},
        files::{cleanup, insert_file, insert_files, patch, write_files},
    },
};

mod downloader;
mod organizer;

static TMP_DIR: &str = "tmp";
static MAX_THREADS: usize = 20;
static MAX_RETRIES: usize = 5;

static DATABASE: &str = "hashes_db";
static TABLE_NAME: &str = "hashes";
static MAX_FILE_COMBINES: usize = 8;

static FILE_SIZE: usize = 1_000_000;
static OUTPUT_DIR: &str = "./hashes";

fn main() -> std::io::Result<()> {
    pretty_env_logger::formatted_timed_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("SB_LOG")
        .init();
    // prepare argument parser
    #[rustfmt::skip]
    let mut parser = Parser::new()
        // general options
        .add_arg("h", "help", "Prints this help prompt", false, false)
        // actions
        .add_arg("f", "fetch", "Fetches the latest files", false, false)
        .add_arg("i", "insert", "Inserts files into db", false, false)
        .add_arg("e", "export", "Exports all hashes from db", false, false)
        .add_arg("if", "insert-file", "Inserts specified file", true, false)
        .add_arg("u", "update", "Fetches and imports", false, false)
        .add_arg("c", "clean", "Clears the temp dir and the database", false, false)
        .add_arg("p", "patch", "Apply a patch file", true, false)
        .add_arg("n", "numerate", "Returns the number of hashes currently in DB", false, false)
        // processing arguments
        .add_arg("t", "tempdir", "Sets the temporary directory; Defaults to ./tmp", true, true)
        .add_arg("d", "database", "Sets the database name; Defaults to hashes_db", true, true)
        .add_arg("mt", "max-threads", "Sets the max download threads; Defaults to 20", true, true)
        .add_arg("mr", "max-retries", "Sets the max download retries; Defaults to 5", true, true)
        .add_arg("mc", "max-combines", "Sets how many files can be combined for inserting; Defaults to 8", true, true)
        .add_arg("tb", "table", "Sets the tablename; Defaults to hashes", true, true)
        // output options
        .add_arg("o", "output", "Sets the output folder; Defaults to ./hashes", true, true)
        .add_arg("l", "length", "The number of lines in output files; Defaults to 1_000_000", true, true);

    // parse arguments
    let _ = parser.parse().is_err_and(|err| {
        error!("{err}");
        exit(-1)
    });

    // if --help was passed output help prompt and exit
    parser.get_parsed_argument_long("help").is_some().then(|| {
        println!("Usage: SB_LOG=[INFO|DEBUG|TRACE] signature-builder [Options...]");
        parser
            .get_arguments()
            .into_iter()
            .for_each(|arg| println!("\t{arg}"));
        exit(0)
    });

    let tmp_dir = parser
        .get_parsed_argument_long("tempdir")
        .and_then(|parsed_argument| parsed_argument.value)
        .unwrap_or(TMP_DIR.to_owned());
    debug!("Set tmp_dir to {tmp_dir}");
    let tmp_dir_arc = Arc::new(Path::new(&tmp_dir).to_owned());

    let database = parser
        .get_parsed_argument_long("database")
        .and_then(|parsed_argument| parsed_argument.value)
        .unwrap_or(DATABASE.to_string());
    debug!("Set database to {database}");

    let max_threads = parser
        .get_parsed_argument_long("max-threads")
        .and_then(|parsed_argument| {
            parsed_argument.value.map(|value| {
                value.parse::<usize>().unwrap_or_else(|err| {
                    error!("Failed to parse {value} for max-threads to usize: {err}");
                    exit(-1)
                })
            })
        })
        .unwrap_or(MAX_THREADS);
    debug!("Set max_threads to {max_threads}");

    let max_retries = parser
        .get_parsed_argument_long("max-retries")
        .and_then(|parsed_argument| {
            parsed_argument.value.map(|value| {
                value.parse::<usize>().unwrap_or_else(|err| {
                    error!("Failed to parse {value} for max-retries to usize: {err}");
                    exit(-1)
                })
            })
        })
        .unwrap_or(MAX_RETRIES);
    debug!("Set max_retries to {max_retries}");

    let max_combines = parser
        .get_parsed_argument_long("max-combines")
        .and_then(|parsed_argument| {
            parsed_argument.value.map(|value| {
                value.parse::<usize>().unwrap_or_else(|err| {
                    error!("Failed to parse {value} for max-combines to usize: {err}");
                    exit(-1)
                })
            })
        })
        .unwrap_or(MAX_FILE_COMBINES);
    debug!("Set max_combines to {max_combines}");

    let table_name = parser
        .get_parsed_argument_long("table")
        .and_then(|parsed_argument| parsed_argument.value)
        .unwrap_or(TABLE_NAME.to_owned());
    debug!("Set table_name to {table_name}");

    let output_dir = parser
        .get_parsed_argument_long("output")
        .and_then(|parsed_argument| parsed_argument.value)
        .unwrap_or(OUTPUT_DIR.to_owned());
    debug!("Set output_dir to {output_dir}");

    let file_size = parser
        .get_parsed_argument_long("length")
        .and_then(|parsed_argument| {
            parsed_argument.value.map(|value| {
                value.parse::<usize>().unwrap_or_else(|err| {
                    error!("Failed to parse {value} for length to usize: {err}");
                    exit(-1)
                })
            })
        })
        .unwrap_or(FILE_SIZE);
    debug!("Set file_size to {file_size}");

    let start_time = std::time::Instant::now();

    parser.get_parsed_argument_long("clean").is_some().then(|| {
        info!("Cleaning database and tmpdir...");
        cleanup(tmp_dir.clone(), database.clone());
    });

    let parsed_arguments = parser.get_parsed_arguments();
    for parsed_argument in parsed_arguments {
        match parsed_argument {
            _ if parsed_argument.long_matches("fetch") => {
                download_all(tmp_dir_arc.clone(), max_threads, max_retries)?
            }
            _ if parsed_argument.long_matches("insert") => insert_files(
                tmp_dir.clone(),
                max_combines,
                database.clone(),
                table_name.clone(),
            )?,
            _ if parsed_argument.long_matches("insert-file") => {
                let file_path = parser
                    .get_parsed_argument_long("insert-file")
                    .and_then(|parsed_argument| parsed_argument.value)
                    .unwrap_or_else(|| {
                        error!("Could not get path for insert-file!");
                        exit(-1)
                    });
                insert_file(file_path, database.clone(), table_name.clone())?;
            }
            _ if parsed_argument.long_matches("update") => {
                download_all(tmp_dir_arc.clone(), max_threads, max_retries)?;
                insert_files(
                    tmp_dir.clone(),
                    max_combines,
                    database.clone(),
                    table_name.clone(),
                )?;
            }
            _ if parsed_argument.long_matches("export") => {
                write_files(
                    output_dir.clone(),
                    file_size,
                    database.clone(),
                    table_name.clone(),
                )?;
            }
            _ if parsed_argument.long_matches("patch") => {
                let file_path = parser
                    .get_parsed_argument_long("patch")
                    .and_then(|parsed_argument| parsed_argument.value)
                    .unwrap_or_else(|| {
                        error!("Could not get path for path!");
                        exit(-1)
                    });
                patch(database.clone(), table_name.clone(), file_path)?;
            }
            _ if parsed_argument.long_matches("numerate") => {
                let database_connection = create_pool(database.clone(), table_name.clone())
                    .map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
                    })?;
                let count =
                    get_hash_count(&database_connection, table_name.clone()).map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
                    })?;
                database_connection.close().map_err(|err| {
                    std::io::Error::new(std::io::ErrorKind::Other, err.1.to_string())
                })?;
                println!("There are currently {count} hashes in DB");
            }
            _ => {}
        }
    }

    info!(
        "Total time was {}s",
        std::time::Instant::now()
            .duration_since(start_time)
            .as_secs_f32()
    );
    Ok(())
}
