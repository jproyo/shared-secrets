use std::collections::HashMap;

use sss_rs::wrapped_sharing::reconstruct;
use sss_wrap::from_secrets;
use sss_wrap::secret::secret::{Metadata, Share, ShareMeta};
use tokio::task::JoinSet;

async fn create_shares(
    api_key: String,
    client_id: String,
    secret: String,
    shares_required: u8,
    shares_to_create: u8,
    servers: HashMap<u8, String>,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    let secret: Vec<u8> = secret.into_bytes();
    let shares = from_secrets(&secret, shares_required, shares_to_create, None).unwrap();

    let meta = &Metadata::new(shares_required, shares_to_create, secret.len());

    let shares_vec: Vec<ShareMeta> = shares
        .into_iter()
        .map(|s| ShareMeta::new(s.into(), meta.clone()))
        .collect::<Vec<_>>();

    let mut tasks = JoinSet::new();
    for s in shares_vec {
        let client_id = client_id.clone();
        let api_key = api_key.clone();
        let url = format!(
            "http://{}/{}/secret",
            servers.get(&s.share.id()).unwrap(),
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
                reqwest::StatusCode::OK => Ok(()),
                _ => {
                    return Err(Box::new(result.error_for_status().unwrap_err()));
                }
            }
        });
    }
    while let Some(res) = tasks.join_next().await {
        let _ = res.unwrap();
    }
    Ok(())
}

async fn get_secret(
    servers: HashMap<u8, String>,
    client_id: String,
    api_key: String,
    shares_required: u8,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let mut shares = Vec::new();

    let mut shares_count = 0;
    for (_, v) in &servers {
        let share = client
            .get(format!("http://{}/{}/share", v, client_id))
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?
            .json::<Share>()
            .await?;
        shares.push(share);
        shares_count += 1;
        if shares_count == shares_required {
            break;
        }
    }
    if shares_count < shares_required {
        Ok(None)
    } else {
        let raw_secret: Vec<Vec<u8>> = shares.into_iter().map(|s| s.into()).collect::<Vec<_>>();
        let secret = reconstruct(raw_secret, false)?;
        Ok(Some(String::from_utf8(secret).unwrap()))
    }
}

#[tokio::test]
async fn test_creating_shares_and_recovering() {
    let servers = [
        (1_u8, "localhost:8000".to_string()),
        (2_u8, "localhost:8001".to_string()),
        (3_u8, "localhost:8002".to_string()),
    ];

    let servers: HashMap<u8, String> = servers.into_iter().collect();

    let client_id = "1".to_string();
    let api_key = "api-key-test".to_string();
    let secret = "my-secret-test".to_string();

    create_shares(
        api_key.clone(),
        client_id.clone(),
        secret.clone(),
        2,
        3,
        servers.clone(),
    )
    .await
    .unwrap();

    let expected = get_secret(servers, client_id, api_key, 2).await.unwrap();
    assert_eq!(expected, Some(secret));
}
