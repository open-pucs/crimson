#![allow(dead_code)]
use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get},
    openapi::OpenApi,
};
use axum::{Extension, Json};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use clap::Parser;
use common::{
    api_documentation::generate_api_docs_and_serve,
    otel_tracing::initialize_tracing_and_wrap_router,
};

use std::{
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
};

use tracing::{Instrument, info};

// use opentelemetry::global::{self, BoxedTracer, ObjectSafeTracerProvider, tracer};

mod api;
mod logic;
mod processing;
mod types;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 14423)]
    port: u16,
}

mod common;
#[tokio::main]
async fn main() -> anyhow::Result<Infallible> {
    let args = Args::parse();

    // initialise our subscriber
    let app_maker = || {
        ApiRouter::new()
            .api_route("/v1/health", get(health))
            .nest("/v1/", api::router())
            .nest("/admin/", admin::router())
    };
    // Add HTTP tracing layer
    // include trace context as header into the response

    let app = initialize_tracing_and_wrap_router(app_maker)?;
    // Spawn background worker to process PDF tasks
    // This worker runs indefinitely
    info!("App Created, spawning background process:");
    tokio::spawn(
        async move {
            processing::worker::start_worker().await;
        }
        .in_current_span(),
    );

    // bind and serve
    let addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), args.port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let Err(serve_err) = generate_api_docs_and_serve(listener, app, "A PDF processing API").await;
    Err(serve_err.into())
}

/// Get health of the API.
async fn health() -> &'static str {
    "Service is Healthy"
}

mod admin {
    use aide::axum::{ApiRouter, IntoApiResponse, routing::get};
    use axum::Json;
    use axum_tracing_opentelemetry::tracing_opentelemetry_instrumentation_sdk;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use tracing::{debug, error, info, warn};

    #[derive(Serialize, Deserialize, JsonSchema)]
    struct ServerInfo {
        name: String,
        version: String,
    }

    /// Expose admin routes
    pub fn router() -> ApiRouter {
        ApiRouter::new().api_route("/info", get(get_server_info))
    }

    /// Get static server info
    async fn get_server_info() -> impl IntoApiResponse {
        let _trace_id_owned = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id()
            .unwrap_or_else(|| "unknown trace id".to_string());
        let example = "test-value";
        debug!(example, "Someone tried to get server info");
        info!(example, "Someone tried to get server info");
        warn!(example, "Someone tried to get server info");
        error!(example, "Someone tried to get server info");
        Json(ServerInfo {
            name: "Crimson".into(),
            version: "0.0".into(),
        })
    }
}
