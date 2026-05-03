use knud::realtime_demo;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    let _ = rustls::crypto::ring::default_provider().install_default();
    realtime_demo().await
}
