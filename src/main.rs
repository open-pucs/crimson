use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get},
    openapi::{Info, OpenApi},
    swagger::Swagger,
};
use axum::{Extension, Json};

use std::net::{Ipv4Addr, SocketAddr};

mod api;
mod logic;
mod processing;
mod types;

// Note that this clones the document on each request.
// To be more efficient, we could wrap it into an Arc,
// or even store it as a serialized string.
async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

// OpenTelemetry and tracing imports
use opentelemetry::sdk::trace::TraceConfig;
use opentelemetry::sdk::Resource;
use opentelemetry::KeyValue as OTelKeyValue;
use opentelemetry_otlp::new_pipeline;
use opentelemetry::runtime::Tokio;
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, Registry};
use tracing_opentelemetry::layer;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize OpenTelemetry OTLP pipeline and tracing subscriber
    let otel_exporter = new_pipeline()
        .with_endpoint(std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").unwrap_or_default())
        .with_trace_config(
            TraceConfig::default().with_resource(Resource::new(vec![
                OTelKeyValue::new("service.name", "crimson-app"),
            ])),
        )
        .install_batch(Tokio)?;
    let tracer = otel_exporter.tracer("crimson_tracer", Some(env!("CARGO_PKG_VERSION")));
    let otel_layer = layer().with_tracer(tracer);
    let fmt_layer = fmt::layer();
    let filter_layer = EnvFilter::from_default_env();
    let subscriber = Registry::default()
        .with(filter_layer)
        .with(fmt_layer)
        .with(otel_layer);
    tracing::subscriber::set_global_default(subscriber)?;

    // Build our application with routes
    let app = ApiRouter::new()
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

    // Ensure all spans are exported before shutdown
    opentelemetry::global::shutdown_tracer_provider();

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