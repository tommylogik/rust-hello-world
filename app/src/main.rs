use axum::{Router, routing::get};

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    // 1. Define your API routes
    let app = Router::new().route("/", get(|| async { "Hello World!" }));
    // 2. Bind a network address
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
        .await
        .unwrap();
    println!("Server running on http://localhost:{port}");

    // 3. Start the server loop
    axum::serve(listener, app).await.unwrap();
}
