use std::collections::HashMap;

use shared_secret_client::conf::settings::Settings;
use sss_wrap::secret::secret::{Metadata, ShareMeta};
use sss_wrap::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::new()?;
    let secret: Vec<u8> = vec![5, 4, 9, 1, 2, 128, 43];
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
