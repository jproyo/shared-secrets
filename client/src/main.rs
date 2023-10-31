use sss_wrap::secret::secret::Share;
use sss_wrap::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shares_required = 2;
    let shares_to_create = 2;
    let secret: Vec<u8> = vec![5, 4, 9, 1, 2, 128, 43];
    let shares = from_secrets(&secret, shares_required, shares_to_create, None).unwrap();

    let shares_vec: Vec<Share> = shares.into_iter().map(|s| s.into()).collect::<Vec<_>>();

    let first_share = shares_vec[0].clone();
    println!("Shares: {:?}", serde_json::to_string(&first_share));

    let client = reqwest::Client::new();
    client
        .post(format!("http://127.0.0.1:8080/{}/secret", "1"))
        .json(&first_share)
        .send()
        .await
        .map(|_| ())
        .map_err(|e| e.into())
}
