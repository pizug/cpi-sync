use clap::Clap;
use crossterm::event::{read, Event};
use jsonschema::{self, Draft, JSONSchema};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use std::{
    env,
    error::Error,
    fs::File,
    io::{stdin, stdout, Read, Write},
};
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
    #[clap(long)]
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    println!("Running cpisync...");
    if !opts.no_input {
        pause();
    }

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

    let client = reqwest::Client::new();

    // let mut authorization: Option<String> = None;

    let mut username: Option<String> = None;
    let mut password: Option<String> = None;

    //get secret from environment variable
    match &config.tenant.credential {
        CredentialInside::SUser(c) => {
            match &c.password_environment_variable {
                Some(varkey) => {
                    match env::var(varkey) {
                        Ok(val) => {
                            password = Some(val);
                        }
                        Err(e) => {
                            println!(
                                "Can not find S-user Pass in environment variable: {}: {}",
                                &varkey, e
                            );
                            // return Err(e.into());
                        }
                    };
                }
                None => (),
            };
        }
        CredentialInside::OauthClientCredentials(c) => {
            match &c.client_secret_environment_variable {
                Some(varkey) => {
                    match env::var(varkey) {
                        Ok(val) => {
                            password = Some(val);
                        }
                        Err(e) => {
                            println!(
                                "Can not find Client Secret environment variable: {}: {}",
                                &varkey, e
                            );
                        }
                    };
                }
                None => (),
            };
        }
    }

    //try to get password from command line
    if !opts.no_input {
        match &password {
            None => {
                let pass = rpassword::prompt_password_stdout("Password: ")?;
                password = Some(pass);
                //println!("Your password is {}", pass);
            }
            _ => {}
        }
    }

    let mut password: String = match password {
        Some(p) => p,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not use any password/secret",
            )
            .into())
        }
    };

    let check_api_url = format!("https://{host}/api/v1/", host = &config.tenant.host);

    //for oauth we need to get the token
    let authorization = match &config.tenant.credential {
        CredentialInside::OauthClientCredentials(c) => {
            let api_token_url = format!(
                "{url}?grant_type=client_credentials",
                url = c.token_endpoint_url
            );
            let auth = basic_auth(&c.client_id, &password);

            let resp = client
                .get(&api_token_url)
                .header("Authorization", auth)
                .send()
                .await?
                .json::<HashMap<String, String>>()
                .await?;

            println!("{:#?}", resp);

            String::new()
        }
        CredentialInside::SUser(c) => basic_auth(&c.username, &password),
    };

    let resp = client
        .get(&check_api_url)
        .header("Authorization", authorization)
        .send()
        .await?;

    let resp_success = &resp.status().is_success();
    let resp_code = resp.status();

    println!("{:#?}, {:#?}", resp_success, resp_code);

    println!("API Check Failed!");

    return Err(std::io::Error::new(std::io::ErrorKind::Other, "API Check Failed!").into());

    if !opts.no_input {
        pause();
    }

    Ok(())
}

fn basic_auth(user: &str, pass: &str) -> String {
    let encoded = base64::encode(format!("{username}:{pass}", username = &user, pass = &pass));
    let authorization = format!("Basic {encoded}", encoded = encoded);
    return authorization;
}
