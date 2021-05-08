use clap::Clap;
use crossterm::event::{read, Event};
use jsonschema::{self, Draft, JSONSchema};
use path_absolutize::Absolutize;
use path_slash::{PathBufExt, PathExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::{
    collections::{HashMap, HashSet},
    env,
    fs::File,
    io::{BufRead, BufReader, Lines, Read, Write},
    iter::FromIterator,
    path::PathBuf,
};
use std::{fs, io::Cursor, ops::Deref};
use zip::read::ZipFile;

//config types

//cli type
#[derive(Clap, Debug)]
#[clap(version = "0.3.0-beta", author = "Fatih.Pense @ pizug.com")]
struct Opts {
    #[clap(short, long, default_value = "./cpi-sync.json")]
    config: String,
    #[clap(long, about = "Disable features that require user input")]
    no_input: bool,
}

fn pause() {
    println!("Press any key to continue...");
    loop {
        // `read()` blocks until an `Event` is available
        match read().unwrap() {
            Event::Key(_) => {
                // println!("{:?}", event);
                break;
            }
            _ => {}
        }
    }
}

async fn run_console(opts: &Opts) -> Result<(), Box<dyn std::error::Error>> {
    println!("Start CPI Sync?");
    if !opts.no_input {
        pause();
    }

    let schema_str = include_str!("../resources/config.schema.json");
    let json_schema: Value = serde_json::from_str(schema_str).unwrap();

    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&json_schema)?;

    let mut config_str = String::new();
    File::open(&opts.config)?.read_to_string(&mut config_str)?;
    // let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let config_json: serde_json::Value = serde_json::from_str(&config_str)?;

    let result = compiled_schema.validate(&config_json);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error);
        }
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "JSON Schema validation error.",
        )
        .into());
    }

    let config: cpi_sync::Config = serde_json::from_str(&config_str)?;

    return cpi_sync::run_with_config(&config, &opts.config, opts.no_input).await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();
    let result = run_console(&opts).await;

    match result {
        Ok(()) => {
            println!("Completed successfully.");
            if !opts.no_input {
                pause();
            }
            return Ok(());
        }
        Err(err) => {
            println!("{:?}", err);
            if !opts.no_input {
                pause();
            }
            return Err(err.into());
        }
    };
}
