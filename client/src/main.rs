use std::collections::HashMap;

use shared_secret_client::conf::settings::Settings;
use sss_wrap::secret::secret::{Metadata, Share, ShareMeta};
use sss_wrap::wrapped_sharing::reconstruct;
use sss_wrap::*;
use structopt::StructOpt;
use strum_macros::EnumString;

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

    let client = reqwest::Client::new();

    for s in shares_vec {
        client
            .post(format!(
                "http://{}/{}/secret",
                map.get(&s.share.id()).unwrap(),
                settings.client_id
            ))
            .json(&s)
            .send()
            .await
            .map(|_| ())
            .unwrap();
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

    for i in 1..=settings.shares_required {
        let share = client
            .get(format!(
                "http://{}/{}/secret",
                map.get(&i).unwrap(),
                settings.client_id,
            ))
            .send()
            .await?
            .json::<Share>()
            .await?;
        shares.push(share);
    }

    let raw_secret: Vec<Vec<u8>> = shares.into_iter().map(|s| s.into()).collect::<Vec<_>>();

    let secret = reconstruct(raw_secret, false)?;

    println!("{}", String::from_utf8(secret).unwrap());

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
