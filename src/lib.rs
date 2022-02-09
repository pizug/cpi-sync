mod config;
use config::*;
use futures::{
    stream::{FuturesUnordered, StreamExt},
    Future,
};
use path_slash::PathBufExt;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    env,
    io::{Read, Write},
    iter::FromIterator,
    path::{Component, Path, PathBuf},
};
use std::{fs, io::Cursor, ops::Deref};

pub use config::Config;

// use rand::seq::SliceRandom;
// use rand::thread_rng;

// response types
#[derive(Serialize, Deserialize, Debug)]
struct APIResponseResult {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Mode")]
    mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct APIResponseD {
    results: Vec<APIResponseResult>,
}
#[derive(Serialize, Deserialize, Debug)]
struct APIResponseRoot {
    d: APIResponseD,
}

// response types: token api

#[derive(Serialize, Deserialize, Debug)]
struct TokenAPIResponseRoot {
    access_token: String,
}

async fn write_artifact(
    package_id: &str,
    artifact_id: &str,
    config: &Config,
    data_dir: &std::path::PathBuf,
    mut respbytes_cursor: Cursor<&[u8]>,
) -> Result<(), Box<dyn std::error::Error>> {
    match config.packages.zip_extraction {
        ZipExtraction::Disabled => {
            let write_dir = data_dir
                .join(&package_id)
                .join(artifact_id.to_string() + ".zip");

            let parent_dir = write_dir.parent().unwrap();
            fs::create_dir_all(parent_dir).unwrap();

            let mut write_dir = fs::File::create(&write_dir).unwrap();
            std::io::copy(&mut respbytes_cursor, &mut write_dir).unwrap();
        }
        ZipExtraction::Enabled => {
            let mut archive = zip::ZipArchive::new(respbytes_cursor).unwrap();

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).unwrap();

                let outpath_str = file.enclosed_name().unwrap().to_str().unwrap();
                let outpath: PathBuf = PathBuf::from_slash(outpath_str);

                // println!(
                //     "data_dir: {:?} , package_id:{:?} , artifact_id: {:?}, outpath: {:?}",
                //     &data_dir, &package_id, &artifact.id, &outpath
                // );
                let write_dir = data_dir.join(&package_id).join(artifact_id).join(outpath);
                // println!("write_dir: {:?} ", &write_dir);

                let parent_dir = write_dir.parent().unwrap();
                fs::create_dir_all(parent_dir).unwrap();
                let mut write_dir = fs::File::create(&write_dir).unwrap();

                match config.packages.prop_comment_removal {
                    PropCommentRemoval::Disabled => {
                        std::io::copy(&mut file, &mut write_dir).unwrap();
                    }
                    PropCommentRemoval::Enabled => {
                        if outpath_str.ends_with("parameters.prop") {
                            let mut prop_content = String::new();
                            file.read_to_string(&mut prop_content)?;

                            let prop_lines: Vec<&str> = prop_content
                                .lines()
                                .filter(|l| !l.starts_with("#"))
                                .collect();

                            for line in prop_lines {
                                write_dir
                                    .write_all(line.as_bytes())
                                    .expect("Couldn't write to file");

                                write_dir.write_all(b"\n").expect("Couldn't write to file");
                            }
                        // write_dir.write_all(lines.as_bytes());
                        } else {
                            std::io::copy(&mut file, &mut write_dir).unwrap();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn download_artifact(
    package_id: String,
    artifact_id: String,
    config: Config,
    data_dir: std::path::PathBuf,
    client: reqwest::Client,
    authorization: String,
    artifact_type: String,
    ignore_error_download: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "- Artifact: {:#?} , from Package: {:#?}",
        artifact_id, package_id
    );

    let api_artifact_payload_url = format!(
        "https://{host}/api/v1/{artifact_type}(Id='{artifact_id}',Version='Active')/$value",
        host = config.tenant.management_host,
        artifact_id = artifact_id,
        artifact_type = artifact_type
    );
    let resp = client
        .get(&api_artifact_payload_url)
        .header("Authorization", authorization)
        .send()
        .await?;

    let resp_success = &resp.status().is_success();
    let resp_code = resp.status();

    if !resp_success {
        println!("Artifact Download Failed!");
        println!("API URL: {}", &api_artifact_payload_url);
        println!("API Response Code: {:#?}", &resp_code);
    }
    if !resp_success && ignore_error_download {
        println!("Ignoring error (Ignore Download Error Option: True)");
    }
    if !resp_success && !ignore_error_download {
        println!("Response Body:");
        let body_text = resp.text().await?;
        println!("{}", &body_text);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "API Artifact Download Failed!",
        )
        .into());
    }

    if *resp_success {
        let respbytes = resp.bytes().await?;
        let respbytes_cursor = Cursor::new(respbytes.deref());

        write_artifact(
            &package_id,
            &artifact_id,
            &config,
            &data_dir,
            respbytes_cursor,
        )
        .await?;
    }
    Ok(())
}

async fn process_package_artifacts(
    package_id: &str,
    artifact_type: &str,
    config: &Config,
    client: &reqwest::Client,
    authorization: &str,
    data_dir: &std::path::PathBuf,
    ignore_error_download: &bool,
) -> Result<
    Vec<impl Future<Output = Result<(), Box<dyn std::error::Error>>>>,
    Box<dyn std::error::Error>,
> {
    let api_package_artifact_list_url = format!(
        "https://{host}/api/v1/IntegrationPackages('{package_id}')/{artifact_type}",
        host = config.tenant.management_host,
        package_id = package_id,
        artifact_type = artifact_type
    );
    let resp = client
        .get(&api_package_artifact_list_url)
        .header("Authorization", authorization)
        .header("Accept", "application/json")
        .send()
        .await?;

    let resp_success = &resp.status().is_success();
    let resp_code = resp.status();

    let body_text = resp.text().await?;

    if !resp_success {
        println!("API Package List Artifacts Failed!");
        println!("Artifact type: {}", &artifact_type);
        println!("API URL: {}", &api_package_artifact_list_url);
        println!("API Response Code: {:#?}", &resp_code);
        println!("Response Body:");
        println!("{}", &body_text);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "API Package List Artifacts Failed!",
        )
        .into());
    }

    let resp_obj: APIResponseRoot = match serde_json::from_slice(body_text.as_bytes()) {
        Ok(api_resp) => api_resp,
        Err(err) => {
            println!("API Package List Artifacts Parse Failed!");
            println!("Artifact type: {}", &artifact_type);
            println!("API URL: {}", &api_package_artifact_list_url);
            println!("API Response Code: {:#?}", &resp_code);
            println!("Response Body:");
            println!("{}", &body_text);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, err).into());
        }
    };

    let mut tasks = Vec::new();
    for artifact in resp_obj.d.results {
        tasks.push(download_artifact(
            package_id.to_owned(),
            artifact.id.to_owned(),
            config.clone(),
            data_dir.clone(),
            client.clone(),
            authorization.to_string(),
            artifact_type.to_string(),
            *ignore_error_download,
        ));
    }
    Ok(tasks)
}

async fn process_package(
    package_id: &str,
    config: &Config,
    client: &reqwest::Client,
    authorization: &str,
    data_dir: &std::path::PathBuf,
    ignore_error_download: &bool,
) -> Result<
    Vec<impl Future<Output = Result<(), Box<dyn std::error::Error>>>>,
    Box<dyn std::error::Error>,
> {
    //remove local package contents before download
    let package_dir = data_dir.join(&package_id);
    remove_dir_all::ensure_empty_dir(&package_dir)?;
    // let _ = fs::remove_dir_all(package_dir);

    println!("Processing Package: {:?}", package_id);

    let mut tasks1 = process_package_artifacts(
        package_id,
        "IntegrationDesigntimeArtifacts",
        config,
        client,
        authorization,
        data_dir,
        ignore_error_download,
    )
    .await?;

    let mut tasks2 = process_package_artifacts(
        package_id,
        "ValueMappingDesigntimeArtifacts",
        config,
        client,
        authorization,
        data_dir,
        ignore_error_download,
    )
    .await?;

    tasks1.append(&mut tasks2);
    Ok(tasks1)
}

async fn get_all_packages(
    config: &Config,
    client: &reqwest::Client,
    authorization: &str,
) -> Result<APIResponseRoot, Box<dyn std::error::Error>> {
    let api_package_list_url = format!(
        "https://{host}/api/v1/IntegrationPackages",
        host = config.tenant.management_host
    );
    let resp = client
        .get(&api_package_list_url)
        .header("Authorization", authorization)
        .header("Accept", "application/json")
        .send()
        .await?;

    let resp_success = &resp.status().is_success();
    let resp_code = resp.status();

    let body_text = resp.text().await?;

    if !resp_success {
        println!("Package List Failed!");
        println!("API URL: {}", &api_package_list_url);
        println!("API Response Code: {:#?}", &resp_code);
        println!("Response Body:");
        println!("{}", &body_text);
        return Err(
            std::io::Error::new(std::io::ErrorKind::Other, "API Package List  Failed!").into(),
        );
    }

    let resp_obj: APIResponseRoot = match serde_json::from_slice(body_text.as_bytes()) {
        Ok(api_resp) => api_resp,
        Err(err) => {
            println!("Package List Failed!");
            println!("API URL: {}", &api_package_list_url);
            println!("API Response Code: {:#?}", &resp_code);
            println!("Response Body:");
            println!("{}", &body_text);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, err).into());
        }
    };
    //println!("{:?}", &resp_obj);

    Ok(resp_obj)
}

pub async fn run_with_config(
    config: &Config,
    config_path: &String,
    no_input: bool,
    ignore_error_download: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("config: {:?}", config);
    //println!("Using input file: {:?}", opts);

    // let mut authorization: Option<String> = None;

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

    let username: String = match &config.tenant.credential {
        CredentialInside::OauthClientCredentials(c) => c.client_id.to_string(),
        CredentialInside::SUser(c) => c.username.to_string(),
    };
    //try to get password from command line
    if !no_input {
        match &password {
            None => {
                let message = format!(
                    "Would you like to enter a password for user: {user} to connect host: {host}?",
                    user = username,
                    host = config.tenant.management_host
                );

                println!("{}", message);

                let pass = rpassword::prompt_password_stdout("Password: ")?;
                password = Some(pass);
                //println!("Your password is {}", pass);
            }
            _ => {}
        }
    }

    let password: String = match password {
        Some(p) => p,
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Could not use any password/secret",
            )
            .into())
        }
    };

    run_with_config_and_password(
        config,
        config_path,
        no_input,
        ignore_error_download,
        &password,
    )
    .await?;

    Ok(())
}

pub async fn run_with_config_and_password(
    config: &Config,
    config_path: &String,
    no_input: bool,
    ignore_error_download: bool,

    password: &String,
) -> Result<(), Box<dyn std::error::Error>> {
    //println!("config: {:?}", config);
    //println!("Using input file: {:?}", opts);

    let now = tokio::time::Instant::now();

    let client = reqwest::Client::new();

    // let mut authorization: Option<String> = None;

    let username: String = match &config.tenant.credential {
        CredentialInside::OauthClientCredentials(c) => c.client_id.to_string(),
        CredentialInside::SUser(c) => c.username.to_string(),
    };
    //try to get password from command line

    let check_api_url = format!(
        "https://{host}/api/v1/",
        host = &config.tenant.management_host
    );

    //for oauth we need to get the token
    let authorization = match &config.tenant.credential {
        CredentialInside::OauthClientCredentials(c) => {
            let api_token_url = format!(
                "{url}?grant_type=client_credentials",
                url = c.token_endpoint_url
            );
            let auth = basic_auth(&c.client_id, &password);

            let resp = client
                .post(&api_token_url)
                .header("Authorization", auth)
                .send()
                .await?;
            println!("Token API status: {:?}", resp.status());
            let respbody = resp.json::<TokenAPIResponseRoot>().await?;

            format!("Bearer {token}", token = respbody.access_token)
        }
        CredentialInside::SUser(c) => basic_auth(&c.username, &password),
    };

    let resp = client
        .get(&check_api_url)
        .header("Authorization", &authorization)
        .send()
        .await?;

    let resp_success = &resp.status().is_success();
    let resp_code = resp.status();

    if !resp_success {
        println!("API First Check Failed!");
        println!("API Response Code: {:#?}", resp_code);
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "API Check Failed!").into());
    } else {
        println!("API First Check Successful.");
    }

    //https://doc.rust-lang.org/std/fs/fn.canonicalize.html

    let normalized_localdir = normalize_path(Path::new(&config.packages.local_dir));
    let mut data_dir = std::path::PathBuf::from(".");
    //config path as starting point:
    data_dir.push(normalize_path(Path::new(&config_path)));
    data_dir = data_dir.parent().unwrap().to_path_buf();

    //localdir can be relative or absolute
    data_dir.push(normalized_localdir);

    tokio::fs::create_dir_all(&data_dir).await?;
    //UNC paths for long windows paths over 260 chars
    data_dir = data_dir.canonicalize().unwrap();

    let api_package_list = get_all_packages(&config, &client, &authorization).await?;

    let mut api_package_set: HashSet<String> = HashSet::new();
    let mut api_package_name_map: HashMap<String, String> = HashMap::new();
    for package in api_package_list.d.results.iter() {
        api_package_set.insert(package.id.to_string());
        match api_package_name_map.entry(package.name.to_string()) {
            std::collections::hash_map::Entry::Occupied(mut e) => {
                e.insert(e.get().clone() + "," + &package.id);
            }
            std::collections::hash_map::Entry::Vacant(e) => {
                e.insert(package.id.to_string());
            }
        };
    }

    let mut operating_package_set: HashSet<String> = HashSet::new();

    for package_rule in config.packages.filter_rules.iter() {
        let mut rule_package_set: HashSet<String> = HashSet::new();
        match package_rule {
            PackageRuleEnum::Regex(rule) => {
                let re = Regex::new(&rule.pattern)?;

                for p in &api_package_set {
                    if re.is_match(&p) {
                        rule_package_set.insert(p.clone());
                    }
                }

                // rule.operation
                match rule.operation {
                    OperationEnum::Include => {
                        //operating_package_set.union(&rule_package_set);
                        operating_package_set.extend(rule_package_set);
                    }
                    OperationEnum::Exclude => {
                        operating_package_set = operating_package_set
                            .difference(&rule_package_set)
                            .cloned()
                            .collect();
                    }
                }
            }
            PackageRuleEnum::Single(rule) => {
                //if single package rule not found in original package list check names and inform.
                if !api_package_set.contains(&rule.id) {
                    println!("Package ID not found: {}", &rule.id);

                    match api_package_name_map.get(&rule.id) {
                        Some(id_for_name) => {
                            println!(
                                "Did you enter the Package name instead of this Package ID?: '{}'",
                                id_for_name
                            );
                        }
                        None => {}
                    }

                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Package ID not found!",
                    )
                    .into());
                }

                match rule.operation {
                    OperationEnum::Include => {
                        operating_package_set.insert(rule.id.clone());
                    }
                    OperationEnum::Exclude => {
                        operating_package_set.remove(&rule.id);
                    }
                }
            }
        }
    }

    let package_list: Vec<String> = Vec::from_iter(operating_package_set);

    println!("Downloading These Packages:");
    println!("{:?}", &package_list);

    let mut futs = FuturesUnordered::new();
    let mut outputs = Vec::new();

    //fetch package artifacts
    for package_id in package_list.iter() {
        futs.push(process_package(
            package_id,
            &config,
            &client,
            &authorization,
            &data_dir,
            &ignore_error_download,
        ));

        if futs.len() >= config.packages.download_worker_count {
            //fail fast
            outputs.push(futs.next().await.unwrap()?);
        }
    }
    // wait for remaining
    while let Some(item) = futs.next().await {
        outputs.push(item?);
    }

    let mut futs2 = FuturesUnordered::new();
    let mut artifact_results = Vec::new();

    // let mut outputs2 = outputs.into_iter().flatten().collect::<Vec<_>>();
    // outputs2.shuffle(&mut thread_rng());
    // for task in outputs2.into_iter() {
    for task in outputs.into_iter().flatten() {
        // task.await;
        futs2.push(task);

        if futs2.len() >= config.packages.download_worker_count {
            //fail fast
            artifact_results.push(futs2.next().await.unwrap()?);
        }
    }

    // wait for remaining
    while let Some(item) = futs2.next().await {
        artifact_results.push(item?);
    }

    println!(
        "Download time elapsed in seconds: {}",
        now.elapsed().as_secs()
    );

    Ok(())
}

fn basic_auth(user: &str, pass: &str) -> String {
    let encoded = base64::encode(format!("{username}:{pass}", username = &user, pass = &pass));
    let authorization = format!("Basic {encoded}", encoded = encoded);
    authorization
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
