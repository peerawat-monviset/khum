mod calc;
mod handler;
mod metrics;
mod scraper;
mod state;

use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "8000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    
    // Initialize AppState (caches static assets on startup)
    let state = Arc::new(state::AppState::new("promos.txt"));
    if let Err(e) = state.load_from_db() {
        eprintln!("Failed to load database: {}", e);
    }

    // Spawn background worker task to scrape and update promo codes periodically
    scraper::spawn_background_scraper(Arc::clone(&state));

    let listener = TcpListener::bind(&addr).await.expect("Failed to bind port");
    println!("Server running on http://{}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let _ = stream.set_nodelay(true); // Disable Nagle's algorithm for low-latency calculations
                let state_clone = Arc::clone(&state);
                tokio::spawn(async move {
                    handler::handle_connection(stream, state_clone).await;
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
