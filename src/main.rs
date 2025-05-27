use aide::{
    IntoApi,
    axum::{ApiRouter, IntoApiResponse, routing::get},
    openapi::{Info, OpenApi},
    swagger::Swagger,
};
use axum::{Extension, Json};
use axum_tracing_opentelemetry::{middleware::OtelAxumLayer, opentelemetry_tracing_layer};

use std::net::{Ipv4Addr, SocketAddr};

use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{Registry, fmt};

// use opentelemetry::global::{self, BoxedTracer, ObjectSafeTracerProvider, tracer};
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing_opentelemetry::layer;

use opentelemetry::trace::TracerProvider;

mod api;
mod logic;
mod processing;
mod types;

use axum::{
    Router,
    body::Bytes,
    extract::MatchedPath,
    http::{HeaderMap, Request},
    response::{Html, Response},
};
use std::time::Duration;
use tokio::net::TcpListener;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::{Span, info_span};
use tracing_subscriber::util::SubscriberInitExt;
// Note that this clones the document on each request.
// To be more efficient, we could wrap it into an Arc,
// or even store it as a serialized string.
async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry tracing and logging
    // Export spans to OTLP endpoint via gRPC
    let otel_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
        .expect("OTEL_EXPORTER_OTLP_ENDPOINT must be set");
    let otlp_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otel_endpoint)
        .build()
        .expect("Failed to create OTLP exporter");
    let otel_provider = SdkTracerProvider::builder()
        .with_simple_exporter(otlp_exporter)
        .build();
    // Create a new OpenTelemetry trace pipeline that prints to stdout
    let stdout_provider: SdkTracerProvider = SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .build();
    let tracer = otel_provider.tracer("crimson");

    // Create a tracing layer with the configured tracer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Use the tracing subscriber `Registry`, or any other subscriber
    // that impls `LookupSpan`
    let subscriber = Registry::default()
        .with(telemetry)
        .with(tracing_subscriber::fmt::layer());

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set tracing subscriber");

    // initialise our subscriber
    let app = ApiRouter::new()
        // Add HTTP tracing layer
        .layer(OtelAxumLayer::default())
        .api_route("/v1/health", get(health))
        .route("/api.json", get(serve_api))
        .route("/swagger", Swagger::new("/api.json").axum_route())
        .nest("/v1/", api::router())
        .nest("/admin/", admin::router());

    // Spawn background worker to process PDF tasks
    // This worker runs indefinitely
    tokio::spawn(async move {
        processing::worker::start_worker().await;
    });

    // bind and serve
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8080);
    info!("Listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let mut api = OpenApi {
        info: Info {
            description: Some("A library for Cheaply Batch Processing PDF's".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };
    axum::serve(
        listener,
        app
            // Generate the documentation.
            .finish_api(&mut api)
            // Expose the documentation to the handlers.
            .layer(Extension(api))
            .into_make_service(),
    )
    .await
    .unwrap();
    // });

    Ok(())
}

/// Get health of the API.
async fn health() -> &'static str {
    "Service is Healthy"
}

mod admin {
    use aide::axum::{ApiRouter, IntoApiResponse, routing::get};
    use axum::Json;
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};
    use tracing::info;

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
        info!("Someone tried to get server info");
        Json(ServerInfo {
            name: "Crimson".into(),
            version: "0.0".into(),
        })
    }
}
