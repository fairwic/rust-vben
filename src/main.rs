use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "rust_vben=info,tower_http=info".into()),
        )
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(5320);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    tracing::info!("starting rust-vben API on http://{addr}");

    let app = rust_vben::build_app()
        .await
        .expect("application should initialize");

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("server should start");
}
