use std::collections::HashMap;

use shared_secret_client::conf::settings::Settings;
use sss_wrap::secret::secret::{Metadata, Share, ShareMeta};
use sss_wrap::wrapped_sharing::reconstruct;
use sss_wrap::*;
use structopt::StructOpt;
use strum_macros::EnumString;
use tokio::task::JoinSet;

#[derive(StructOpt, EnumString)]
enum Command {
    #[strum(serialize = "create")]
    Create,
    #[strum(serialize = "get")]
    Get,
}

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    secret: Option<String>,
    #[structopt(short, long)]
    command: Command,
}

async fn send_secret(
    settings: &Settings,
    secret: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let secret: Vec<u8> = secret.into_bytes();
    let shares = from_secrets(
        &secret,
        settings.shares_required,
        settings.shares_to_create,
        None,
    )
    .unwrap();

    let meta = &Metadata::new(
        settings.shares_required,
        settings.shares_to_create,
        secret.len(),
    );

    let shares_vec: Vec<ShareMeta> = shares
        .into_iter()
        .map(|s| ShareMeta::new(s.into(), meta.clone()))
        .collect::<Vec<_>>();

    let map: HashMap<u8, String> = settings
        .servers
        .iter()
        .map(|x| (x.id, x.addr.clone()))
        .collect();

    let mut tasks = JoinSet::new();
    for s in shares_vec {
        let client_id = settings.client_id.clone();
        let api_key = settings.api_key.clone();
        let url = format!(
            "http://{}/api/{}/secret",
            map.get(&s.share.id()).unwrap(),
            client_id
        );
        tasks.spawn(async move {
            let client = reqwest::Client::new();
            let result = client
                .post(url.clone())
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&s)
                .send()
                .await?;
            match result.status() {
                reqwest::StatusCode::OK => {
                    println!("Share sent to server {:?}", url);
                    Ok(())
                }
                _ => {
                    eprintln!("Error sending share to server {:?}", url);
                    return Err(Box::new(result.error_for_status().unwrap_err()));
                }
            }
        });
    }
    while let Some(res) = tasks.join_next().await {
        match res {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{:?}", e);
            }
        }
    }

    Ok(())
}

async fn get_secret(settings: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    let map: HashMap<u8, String> = settings
        .servers
        .iter()
        .map(|x| (x.id, x.addr.clone()))
        .collect();

    let client = reqwest::Client::new();

    let mut shares = Vec::new();

    let mut shares_count = 0;
    'outer: loop {
        for (_, v) in &map {
            let share = client
                .get(format!("http://{}/api/{}/share", v, settings.client_id,))
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", settings.api_key))
                .send()
                .await;
            match share {
                Err(_) => {
                    eprintln!("Error getting share from server");
                    break 'outer;
                }
                Ok(share) => {
                    if share.status() == reqwest::StatusCode::OK {
                        let share = share.json::<Share>().await;
                        let share = match share {
                            Err(_) => {
                                eprintln!("Error getting share from server");
                                break 'outer;
                            }
                            Ok(share) => share,
                        };
                        shares.push(share);
                        shares_count += 1;
                        if shares_count == settings.shares_required {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }
    if shares_count < settings.shares_required {
        eprintln!("Not enough shares to reconstruct secret");
        return Ok(());
    } else {
        let raw_secret: Vec<Vec<u8>> = shares.into_iter().map(|s| s.into()).collect::<Vec<_>>();

        let secret = reconstruct(raw_secret, false);

        match secret {
            Err(_) => {
                eprintln!("Error reconstructing secret");
                return Ok(());
            }
            Ok(secret) => {
                println!("SECRET ===> {}", String::from_utf8(secret).unwrap());
            }
        };
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let options = Options::from_args();
    let settings = Settings::new()?;
    match options.command {
        Command::Create => {
            let secret = options.secret.ok_or("Secret is required")?;
            send_secret(&settings, secret).await
        }
        Command::Get => get_secret(&settings).await,
    }
}
