use api::{HostsConfig, SingleHostConfig};
use axum::response::Response;
use clap::Parser;
mod args;
mod pdns_api;

use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let args = args::Args::parse();

    let app = axum::Router::new()
        .route("/", axum::routing::get(root))
        .route("/update", axum::routing::post(update))
        .with_state(args);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::debug!("listening on {}", addr);
    axum::serve(
        TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}

async fn root() -> &'static str {
    "Hello, World! https://github.com/danya02/powerdns-dyndns-agent"
}

async fn update(
    axum::extract::State(args): axum::extract::State<args::Args>,
    headers: axum::http::HeaderMap,
    payload_body: axum::body::Bytes,
) -> Response {
    use p256::ecdsa::signature::Verifier;

    let payload = match serde_json::from_slice::<api::UpdateRequest>(&payload_body) {
        Ok(payload) => payload,
        Err(e) => {
            tracing::error!("Failed to parse payload: {}", e);
            return axum::response::IntoResponse::into_response((
                axum::http::StatusCode::BAD_REQUEST,
                format!("Failed to parse payload: {}", e),
            ));
        }
    };

    let Some(signature_json) = headers.get("X-Signature") else {
        tracing::error!("Missing X-Signature header");
        return axum::response::IntoResponse::into_response((
            axum::http::StatusCode::UNAUTHORIZED,
            "Missing X-Signature header",
        ));
    };
    let signature: p256::ecdsa::Signature =
        match serde_json::from_str(signature_json.to_str().unwrap()) {
            Ok(signature) => signature,
            Err(e) => {
                tracing::error!("Failed to parse X-Signature header: {}", e);
                return axum::response::IntoResponse::into_response((
                    axum::http::StatusCode::UNAUTHORIZED,
                    format!("Failed to parse X-Signature header: {}", e),
                ));
            }
        };

    tracing::debug!("Parsed X-Signature header: {:?}", signature);

    let verifying_key = match p256::ecdsa::VerifyingKey::from_sec1_bytes(&payload.verifying_key) {
        Ok(verifying_key) => verifying_key,
        Err(e) => {
            tracing::error!("Failed to parse verifying key: {}", e);
            return axum::response::IntoResponse::into_response((
                axum::http::StatusCode::UNAUTHORIZED,
                format!("Failed to parse verifying key: {}", e),
            ));
        }
    };

    if let Err(e) = verifying_key.verify(&payload_body, &signature) {
        tracing::error!("Failed to verify signature: {}", e);
        return axum::response::IntoResponse::into_response((
            axum::http::StatusCode::UNAUTHORIZED,
            format!("Failed to verify signature: {}", e),
        ));
    }

    let hosts = match tokio::fs::read_to_string(&args.hosts_file).await {
        Ok(content) => {
            tracing::debug!("Read hosts file: {}", args.hosts_file);
            content
        }
        Err(e) => {
            tracing::error!("Failed to read hosts file: {}", args.hosts_file);
            return axum::response::IntoResponse::into_response((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
            ));
        }
    };

    let hosts: HostsConfig = match serde_json::from_str(&hosts) {
        Ok(hosts) => hosts,
        Err(e) => {
            tracing::error!("Failed to parse hosts file: {}", e);
            return axum::response::IntoResponse::into_response((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                e.to_string(),
            ));
        }
    };

    let mut is_ok = false;
    for host in hosts.hosts.iter() {
        if host.verifying_key == payload.verifying_key {
            if host.allowed_hosts.contains(&payload.name) {
                tracing::debug!("This verifying key is allowed to update {}", payload.name);
                is_ok = true;
                break;
            }
        }
    }

    if !is_ok {
        tracing::error!(
            "The verifying key is not allowed to update {}",
            payload.name
        );
        tracing::error!("If you'd like to allow this, add the following to your hosts file:");
        let potential_config = SingleHostConfig {
            verifying_key: payload.verifying_key.clone(),
            allowed_hosts: vec![payload.name.clone()],
        };
        tracing::error!("{}", serde_json::to_string(&potential_config).unwrap());

        return axum::response::IntoResponse::into_response((
            axum::http::StatusCode::UNAUTHORIZED,
            format!(
                "Your verifying key is not allowed to update {}",
                payload.name
            ),
        ));
    }

    tracing::debug!("hosts file contents: {:?}", hosts);

    let result = pdns_api::update_a_record(
        &args.root_url,
        &args.api_key,
        &args.zone,
        &payload.name,
        &payload.content,
    )
    .await;

    match result {
        Ok(_) => axum::response::IntoResponse::into_response(()),
        Err(e) => axum::response::IntoResponse::into_response((
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            e.to_string(),
        )),
    }
}
