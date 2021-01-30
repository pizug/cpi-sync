use std::{error::Error, fs::File, io::BufReader};

use clap::Clap;
use jsonschema::{self, Draft, JSONSchema};
use serde_json::{self, Value};

#[derive(Clap, Debug)]
#[clap(version = "0.1", author = "Fatih Pense @ pizug.com")]
struct Opts {
    #[clap(short, long, default_value = "cpi-sync.json")]
    config: String,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opts = Opts::parse();

    let schema_str = include_str!("../resources/config-schema.json");
    let json_schema: Value = serde_json::from_str(schema_str).unwrap();

    let compiled_schema = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&json_schema)?;

    let file = File::open(&opts.config)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `User`.
    let config_json: serde_json::Value = serde_json::from_reader(reader)?;

    let result = compiled_schema.validate(&config_json);
    if let Err(errors) = result {
        for error in errors {
            println!("Validation error: {}", error);
        }
    }

    //println!("Using input file: {:?}", opts);

    Ok(())
}
