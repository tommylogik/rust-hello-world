use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    // 1. Define your API routes
    let app = Router::new().route("/", get(|| async { "Hello World!" }));
    // 2. Bind a network address
    let binding_address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(binding_address.clone())
        .await
        .unwrap();

    if cfg!(debug_assertions) {
        println!("Server running on http://localhost:{port}");
    } else {
        println!("Listening on {}", binding_address);
    }

    // 3. Start the server loop
    axum::serve(listener, app).await.unwrap();
}
