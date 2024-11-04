use clap::Parser;
use p256::ecdsa::signature::Signer;
use p256::ecdsa::{self, SigningKey};
mod args;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = args::Args::parse();

    let signing_key = get_keys(&args.private_key).await;

    let value = if let Some(ip) = args.ip_address {
        tracing::info!("Using manually provided IP address: {}", ip);
        ip.clone()
    } else {
        tracing::info!("Getting current IP address");
        let ip = local_ip_address::local_ip().unwrap();
        tracing::info!("Local IP address: {}", ip);
        ip.to_string()
    };

    let client = reqwest::Client::new();
    let url = format!("{}/update", args.url);
    let verifying_key = signing_key.verifying_key();
    let request = api::UpdateRequest {
        name: args.hostname,
        content: value,
        verifying_key: verifying_key.to_sec1_bytes().to_vec(),
    };
    tracing::info!("Sending request to {url}: {:?}", request);

    let request = serde_json::to_vec(&request).unwrap();

    let signature: ecdsa::Signature = signing_key.sign(&request);

    let signature_json = serde_json::to_string(&signature).unwrap();

    tracing::info!("Signature: {}", signature_json);

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-Signature", signature_json)
        .body(request)
        .send()
        .await
        .unwrap();

    if !response.status().is_success() {
        tracing::error!(
            "Failed to send request: {:?}",
            response.text().await.unwrap()
        );
        return;
    }

    tracing::info!("Done!");
}

async fn get_keys(private_key: &str) -> p256::ecdsa::SigningKey {
    match tokio::fs::read(private_key).await {
        Ok(private_key_text) => {
            tracing::info!("Found key file, trying to read it");
            SigningKey::from_slice(&private_key_text).unwrap()
        }
        Err(why) => {
            tracing::info!("Failed to read key file: {}", why);
            tracing::info!("Generating new keys");
            let signing_key = p256::ecdsa::SigningKey::random(&mut rand::rngs::OsRng);
            let signing_key_text = signing_key.to_bytes();
            tokio::fs::write(private_key, &signing_key_text)
                .await
                .unwrap();
            signing_key
        }
    }
}
