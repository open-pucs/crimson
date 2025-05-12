use std::net::{Ipv4Addr, SocketAddr};
use axum::{Router, routing::get};
use hyper::Server;

mod docs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build our application with routes
    let app = Router::new()
        .route("/api/health", get(health).head(health))
        .nest("/api/docs", docs::router())
        .nest("/api/admin", admin::router());

    // bind and serve
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), 8080);
    println!("Listening on http://{}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

/// Get health of the API.
async fn health() -> &'static str {
    "Service is Healthy"
}

mod admin {
    use axum::{Router, routing::get, Json};
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize)]
    struct ServerInfo {
        name: String,
        version: String,
    }

    /// Expose admin routes
    pub fn router() -> Router {
        Router::new().route("/info", get(get_server_info))
    }

    /// Get static server info
    async fn get_server_info() -> Json<ServerInfo> {
        Json(ServerInfo {
            name: "Crimson".into(),
            version: "0.0".into(),
        })
    }
}