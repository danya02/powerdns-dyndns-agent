use clap::Parser;

mod args;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let args = args::Args::parse();

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
    let request = api::UpdateRequest {
        name: args.hostname,
        content: value,
    };
    tracing::info!("Sending request to {url}: {:?}", request);
    client.post(url).json(&request).send().await.unwrap();

    tracing::info!("Done!");
}
