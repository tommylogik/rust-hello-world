use axum::{
    Json, Router,
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ApiResponse {
    pub message: String,
}

pub fn get_subscriber<Sink>(
    app_name: String,
    default_env_filter: String,
    sink: Sink,
) -> impl tracing::Subscriber + Send + Sync
where
    Sink: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_env_filter));
    let formatting_layer = BunyanFormattingLayer::new(app_name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_tracing(app_name: &str, default_env_filter: &str) {
    let subscriber = get_subscriber(app_name.into(), default_env_filter.into(), std::io::stdout);
    let _ = subscriber.try_init();
}

pub fn app() -> Router {
    Router::new()
        .route("/", get(hello_world))
        .route("/api", get(api_hello_world))
        .route("/ws", get(ws_handler))
        .route("/websocket", get(ws_handler))
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    init_tracing("hello-world", "info");

    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    // 1. Define your API routes
    let app = app();

    // 2. Bind a network address
    let binding_address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(binding_address.clone())
        .await
        .unwrap();

    if cfg!(debug_assertions) {
        tracing::info!("Server running on http://localhost:{port}");
    } else {
        tracing::info!("Listening on {}", binding_address);
    }

    // 3. Start the server loop
    axum::serve(listener, app).await.unwrap();
}

async fn hello_world() -> &'static str {
    "Hello World!"
}

async fn api_hello_world() -> Json<ApiResponse> {
    Json(ApiResponse {
        message: "Hello World!".to_string(),
    })
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    if socket
        .send(Message::Text("Hello World!".into()))
        .await
        .is_err()
    {
        return;
    }

    while let Some(Ok(_msg)) = socket.recv().await {
        // Keep the connection open until client closes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::StreamExt;
    use std::net::SocketAddr;
    use tokio_tungstenite::connect_async;

    async fn spawn_test_server() -> SocketAddr {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let app = app();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        addr
    }

    #[test]
    fn test_bunyan_log_formatting() {
        use std::io::Write;
        use std::sync::{Arc, Mutex};

        #[derive(Clone)]
        struct TestWriter(Arc<Mutex<Vec<u8>>>);

        impl Write for TestWriter {
            fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().write(buf)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                self.0.lock().unwrap().flush()
            }
        }

        impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for TestWriter {
            type Writer = TestWriter;
            fn make_writer(&'a self) -> Self::Writer {
                self.clone()
            }
        }

        let buffer = Arc::new(Mutex::new(Vec::new()));
        let writer = TestWriter(buffer.clone());

        let subscriber = get_subscriber("test-app".into(), "info".into(), writer);
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("Test bunyan log message");
        });

        let output = String::from_utf8(buffer.lock().unwrap().clone()).unwrap();
        assert!(!output.is_empty(), "Log output should not be empty");

        let parsed: serde_json::Value =
            serde_json::from_str(output.trim()).expect("Output must be valid JSON");
        assert_eq!(parsed["name"], "test-app");
        assert_eq!(parsed["msg"], "Test bunyan log message");
        assert_eq!(parsed["v"], 0);
        assert_eq!(parsed["level"], 30);
        assert!(parsed.get("time").is_some());
        assert!(parsed.get("pid").is_some());
        assert!(parsed.get("hostname").is_some());
    }

    #[tokio::test]
    async fn test_get_root() {
        let addr = spawn_test_server().await;
        let response = simple_http_get(&format!("http://{}/", addr)).await;
        assert_eq!(response, "Hello World!");
    }

    #[tokio::test]
    async fn test_get_api() {
        let addr = spawn_test_server().await;
        let response = simple_http_get(&format!("http://{}/api", addr)).await;
        let parsed: ApiResponse =
            serde_json::from_str(&response).expect("Failed to parse JSON response");
        assert_eq!(
            parsed,
            ApiResponse {
                message: "Hello World!".to_string()
            }
        );
    }

    #[tokio::test]
    async fn test_websocket_ws() {
        let addr = spawn_test_server().await;
        let url = format!("ws://{}/ws", addr);
        let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

        if let Some(msg) = ws_stream.next().await {
            let msg = msg.expect("Failed to get message");
            assert_eq!(msg.to_text().unwrap(), "Hello World!");
        } else {
            panic!("No message received on websocket");
        }
    }

    #[tokio::test]
    async fn test_websocket_route() {
        let addr = spawn_test_server().await;
        let url = format!("ws://{}/websocket", addr);
        let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

        if let Some(msg) = ws_stream.next().await {
            let msg = msg.expect("Failed to get message");
            assert_eq!(msg.to_text().unwrap(), "Hello World!");
        } else {
            panic!("No message received on websocket");
        }
    }

    async fn simple_http_get(url: &str) -> String {
        let url_no_scheme = url.strip_prefix("http://").unwrap();
        let (host_port, path) = match url_no_scheme.split_once('/') {
            Some((hp, p)) => (hp, format!("/{}", p)),
            None => (url_no_scheme, "/".to_string()),
        };

        let mut stream = tokio::net::TcpStream::connect(host_port).await.unwrap();
        let request = format!(
            "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n\r\n",
            path, host_port
        );
        tokio::io::AsyncWriteExt::write_all(&mut stream, request.as_bytes())
            .await
            .unwrap();

        let mut response = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut response)
            .await
            .unwrap();
        let response_str = String::from_utf8(response).unwrap();
        let body = response_str.split("\r\n\r\n").nth(1).unwrap();
        body.to_string()
    }
}
