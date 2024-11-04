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
    axum::Json(payload): axum::Json<api::UpdateRequest>,
) -> Response {
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
