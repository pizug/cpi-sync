use std::{env, error::Error, fs::File, io::Read};

use clap::Clap;
use jsonschema::{self, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};

use std::collections::HashMap;
#[derive(Serialize, Deserialize, Debug)]
struct Package {
    id: String,
    local: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CredentialSUser {
    username: String,
    password_environment_variable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CredentialOauthClientCredentials {
    client_id: String,
    token_endpoint_url: String,
    client_secret_environment_variable: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
enum CredentialInside {
    #[serde(rename = "oauth_client_credentials")]
    OauthClientCredentials(CredentialOauthClientCredentials),
    #[serde(rename = "s_user")]
    SUser(CredentialSUser),
}

#[derive(Serialize, Deserialize, Debug)]
struct Tenant {
    host: String,
    credential: CredentialInside,
    // credential: CredentialInside,
}

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    cpisync: String,
    tenant: Tenant,
    packages: Vec<Package>,
}

#[derive(Clap, Debug)]
#[clap(version = "0.1.0", author = "Fatih Pense @ pizug.com")]
struct Opts {
    #[clap(short, long, default_value = "cpi-sync.json")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let schema_str = include_str!("../resources/config-schema.json");
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
    }

    let config: Config = serde_json::from_str(&config_str)?;

    // println!("config: {:?}", config);
    //println!("Using input file: {:?}", opts);

    &config.tenant.host;

    // let passvalue2: String = env::var(key)?;

    let str2 = format!("https://{host}/api/v1/", host = &config.tenant.host);

    println!("Value: {:?}", str2);
    let client = reqwest::Client::new();

    let mut authorization: Option<String> = None;

    match config.tenant.credential {
        CredentialInside::SUser(c) => {
            match &c.password_environment_variable {
                Some(varkey) => {
                    match env::var(varkey) {
                        Ok(val) => {
                            let encoded = base64::encode(format!(
                                "{username}:{pass}",
                                username = &c.username,
                                pass = &val
                            ));
                            authorization = Some(format!("Basic {encoded}", encoded = encoded));
                        }
                        Err(e) => {
                            println!("Can not find environment variable: {}: {}", &varkey, e);
                            return Err(e.into());
                        }
                    };
                }
                None => (),
            };
        }
        CredentialInside::OauthClientCredentials(c) => {}
    }
    match &authorization {
        Some(auth) => {
            let resp = client
                .get(&str2)
                .header("Authorization", auth)
                .send()
                .await?;

            let resp2 = resp.status().is_success();
            println!("{:#?}", resp2);
        }
        None => {
            println!("Could not retrieve password/secret.")
        }
    }

    Ok(())
}
