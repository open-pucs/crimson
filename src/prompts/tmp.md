Currently our system for otel traces is working, they are successfully being ingested to our lgtm stack (loki + grafana + tempo + mimir), however so far everything only seems to be visible in the tracing database, tempo. Would it be possible to modify the system to ingest logs using opentelemetry in addition to just the traces?



I am hitting this endpoint as a test:
```rs

    async fn get_server_info() -> impl IntoApiResponse {
        let trace_id_owned = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id()
            .unwrap_or_else(|| "unknown trace id".to_string());
        let trace_id = &trace_id_owned;
        debug!(trace_id, "Someone tried to get server info");
        info!(trace_id, "Someone tried to get server info");
        warn!(trace_id, "Someone tried to get server info");
        error!(trace_id, "Someone tried to get server info");
        Json(ServerInfo {
            name: "Crimson".into(),
            version: "0.0".into(),
        })
    }
```

Here is what tracing is displaying on the standard out:
```
  7153.231954210s  INFO crimson::admin: Someone tried to get server info, trace_id: "1d6ceb4c7aab911248d46ccf1806bd2c"
    at src/main.rs:193 on tokio-runtime-worker
    in otel::tracing::HTTP request with http.request.method: GET, network.protocol.version: 1.1, server.address: "localhost:8080", user_agent.original: "xh/0.24.0", url.path: "/admin/info", url.scheme: "", otel.name: GET, otel.kind: Server, span.type: "web", http.route: "/admin/info", otel.name: "GET /admin/info"

  7153.231993915s  WARN crimson::admin: Someone tried to get server info, trace_id: "1d6ceb4c7aab911248d46ccf1806bd2c"
    at src/main.rs:194 on tokio-runtime-worker
    in otel::tracing::HTTP request with http.request.method: GET, network.protocol.version: 1.1, server.address: "localhost:8080", user_agent.original: "xh/0.24.0", url.path: "/admin/info", url.scheme: "", otel.name: GET, otel.kind: Server, span.type: "web", http.route: "/admin/info", otel.name: "GET /admin/info"

  7153.232025594s ERROR crimson::admin: Someone tried to get server info, trace_id: "1d6ceb4c7aab911248d46ccf1806bd2c"
    at src/main.rs:195 on tokio-runtime-worker
    in otel::tracing::HTTP request with http.request.method: GET, network.protocol.version: 1.1, server.address: "localhost:8080", user_agent.original: "xh/0.24.0", url.path: "/admin/info", url.scheme: "", otel.name: GET, otel.kind: Server, span.type: "web", http.route: "/admin/info", otel.name: "GET /admin/info"
```


And all the traces are ingesting perfectly:

```json
{"batches":[{"resource":{"attributes":[{"key":"telemetry.sdk.language","value":{"stringValue":"rust"}},{"key":"telemetry.sdk.name","value":{"stringValue":"opentelemetry"}},{"key":"telemetry.sdk.version","value":{"stringValue":"0.29.0"}},{"key":"service.name","value":{"stringValue":"crimson"}}],"droppedAttributesCount":0},"instrumentationLibrarySpans":[{"spans":[{"traceId":"13124c74f167fc3095a0bb56dbcab01c","spanId":"c203880b06cc0dc9","parentSpanId":"","traceState":"","name":"GET /admin/info","kind":"SPAN_KIND_SERVER","startTimeUnixNano":1748442169307227400,"endTimeUnixNano":1748442169308466000,"attributes":[{"key":"code.filepath","value":{"stringValue":"/home/nicole/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tracing-opentelemetry-instrumentation-sdk-0.28.1/src/http/http_server.rs"}},{"key":"code.namespace","value":{"stringValue":"tracing_opentelemetry_instrumentation_sdk::http::http_server"}},{"key":"code.lineno","value":{"intValue":13}},{"key":"thread.id","value":{"intValue":11}},{"key":"thread.name","value":{"stringValue":"tokio-runtime-worker"}},{"key":"http.request.method","value":{"stringValue":"GET"}},{"key":"network.protocol.version","value":{"stringValue":"1.1"}},{"key":"server.address","value":{"stringValue":"localhost:8080"}},{"key":"user_agent.original","value":{"stringValue":"xh/0.24.0"}},{"key":"url.path","value":{"stringValue":"/admin/info"}},{"key":"url.scheme","value":{"stringValue":""}},{"key":"span.type","value":{"stringValue":"web"}},{"key":"http.route","value":{"stringValue":"/admin/info"}},{"key":"http.response.status_code","value":{"stringValue":"200"}},{"key":"busy_ns","value":{"intValue":815982}},{"key":"idle_ns","value":{"intValue":446067}}],"droppedAttributesCount":0,"droppedEventsCount":0,"droppedLinksCount":0,"status":{"code":2,"message":""},"events":[{"timeUnixNano":1748442169308176000,"attributes":[{"key":"level","value":{"stringValue":"INFO"}},{"key":"target","value":{"stringValue":"crimson::admin"}},{"key":"trace_id","value":{"stringValue":"13124c74f167fc3095a0bb56dbcab01c"}},{"key":"code.filepath","value":{"stringValue":"src/main.rs"}},{"key":"code.namespace","value":{"stringValue":"crimson::admin"}},{"key":"code.lineno","value":{"intValue":193}}],"droppedAttributesCount":0,"name":"Someone tried to get server info"},{"timeUnixNano":1748442169308281600,"attributes":[{"key":"level","value":{"stringValue":"WARN"}},{"key":"target","value":{"stringValue":"crimson::admin"}},{"key":"trace_id","value":{"stringValue":"13124c74f167fc3095a0bb56dbcab01c"}},{"key":"code.filepath","value":{"stringValue":"src/main.rs"}},{"key":"code.namespace","value":{"stringValue":"crimson::admin"}},{"key":"code.lineno","value":{"intValue":194}}],"droppedAttributesCount":0,"name":"Someone tried to get server info"},{"timeUnixNano":1748442169308324400,"attributes":[{"key":"level","value":{"stringValue":"ERROR"}},{"key":"target","value":{"stringValue":"crimson::admin"}},{"key":"trace_id","value":{"stringValue":"13124c74f167fc3095a0bb56dbcab01c"}},{"key":"code.filepath","value":{"stringValue":"src/main.rs"}},{"key":"code.namespace","value":{"stringValue":"crimson::admin"}},{"key":"code.lineno","value":{"intValue":195}}],"droppedAttributesCount":0,"name":"Someone tried to get server info"}]}]}]}]}
```


But when I try to figure out what logs are associated with this trace it isnt giving me any information.

After you have identified the source of the problem could you go ahead and explain what might be going wrong and explain how you could fix it by ingesting all the logs to opentelemetry along with the traces?

All the relavent code for this is right here:

```rs

use aide::{
    axum::{ApiRouter, IntoApiResponse, routing::get},
    openapi::{Info, OpenApi},
    swagger::Swagger,
};
use axum::{Extension, Json};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use otel_bs::init_subscribers_and_loglevel;

use std::net::{Ipv4Addr, SocketAddr};

use tracing::{Subscriber, info};

// use opentelemetry::global::{self, BoxedTracer, ObjectSafeTracerProvider, tracer};

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

mod otel_bs {

    use init_tracing_opentelemetry::{
        init_propagator, //stdio,
        otlp,
        resource::DetectResource,
        tracing_subscriber_ext::{TracingGuard, build_logger_text},
    };
    use opentelemetry::trace::TracerProvider;
    use opentelemetry_sdk::trace::{SdkTracerProvider, Tracer};
    use tracing::{Subscriber, info, level_filters::LevelFilter};
    use tracing_opentelemetry::OpenTelemetryLayer;
    use tracing_subscriber::{
        Layer, filter::EnvFilter, layer::SubscriberExt, registry::LookupSpan, reload::Error,
    };

    pub fn build_loglevel_filter_layer() -> tracing_subscriber::filter::EnvFilter {
        // filter what is output on log (fmt)
        // std::env::set_var("RUST_LOG", "warn,axum_tracing_opentelemetry=info,otel=debug");

        // TLDR: Unsafe because its not thread safe, however we arent using it in that context so
        // everything should be fine: https://doc.rust-lang.org/std/env/fn.set_var.html#safety
        unsafe {
            std::env::set_var(
                "RUST_LOG",
                format!(
                    // `otel::tracing` should be a level trace to emit opentelemetry trace & span
                    // `otel::setup` set to debug to log detected resources, configuration read and infered
                    "{},otel::tracing=trace,otel=debug",
                    std::env::var("OTEL_LOG_LEVEL").unwrap_or_else(|_| "info".to_string())
                ),
            );
        }
        EnvFilter::from_default_env()
    }

    pub fn build_otel_layer<S>()
    -> anyhow::Result<(OpenTelemetryLayer<S, Tracer>, SdkTracerProvider)>
    where
        S: Subscriber + for<'a> LookupSpan<'a>,
    {
        use opentelemetry::global;
        let otel_rsrc = DetectResource::default()
            .with_fallback_service_name(env!("CARGO_PKG_NAME"))
            // .with_fallback_service_version(env!("CARGO_PKG_VERSION"))
            .build();
        let tracer_provider = otlp::init_tracerprovider(otel_rsrc, otlp::identity)?;

        // to not send trace somewhere, but continue to create and propagate,...
        // then send them to `axum_tracing_opentelemetry::stdio::WriteNoWhere::default()`
        // or to `std::io::stdout()` to print
        //
        // let otel_tracer = stdio::init_tracer(
        //     otel_rsrc,
        //     stdio::identity::<stdio::WriteNoWhere>,
        //     stdio::WriteNoWhere::default(),
        // )?;
        init_propagator()?;
        let layer = tracing_opentelemetry::layer()
            .with_error_records_to_exceptions(true)
            .with_tracer(tracer_provider.tracer(""));
        global::set_tracer_provider(tracer_provider.clone());
        Ok((layer, tracer_provider))
    }

    pub fn init_subscribers_and_loglevel() -> anyhow::Result<SdkTracerProvider> {
        //setup a temporary subscriber to log output during setup
        let subscriber = tracing_subscriber::registry()
            .with(build_loglevel_filter_layer())
            .with(build_logger_text());
        let _guard = tracing::subscriber::set_default(subscriber);
        info!("init logging & tracing");

        let (layer, guard) = build_otel_layer()?;

        let subscriber = tracing_subscriber::registry()
            .with(layer)
            .with(build_loglevel_filter_layer())
            .with(build_logger_text());
        tracing::subscriber::set_global_default(subscriber)?;
        Ok(guard)
    }
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = init_subscribers_and_loglevel()?;

    info!("Tracing Subscriber is up and running, trying to create app");
    // initialise our subscriber
    let app = ApiRouter::new()
        .api_route("/v1/health", get(health))
        .route("/api.json", get(serve_api))
        .route("/swagger", Swagger::new("/api.json").axum_route())
        .nest("/v1/", api::router())
        .nest("/admin/", admin::router())
        // Add HTTP tracing layer
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default());

    // Spawn background worker to process PDF tasks
    // This worker runs indefinitely
    info!("App Created, spawning background process:");
    tokio::spawn(async move {
        processing::worker::start_worker().await;
    });

    // bind and serve
    let addr = SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8080);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    info!("Listening on http://{}", addr);
    let mut api = OpenApi {
        info: Info {
            description: Some("A library for Cheaply Batch Processing PDF's".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };
    info!("Initialized OpenAPI");
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
        let trace_id_owned = tracing_opentelemetry_instrumentation_sdk::find_current_trace_id()
            .unwrap_or_else(|| "unknown trace id".to_string());
        let trace_id = &trace_id_owned;
        debug!(trace_id, "Someone tried to get server info");
        info!(trace_id, "Someone tried to get server info");
        warn!(trace_id, "Someone tried to get server info");
        error!(trace_id, "Someone tried to get server info");
        Json(ServerInfo {
            name: "Crimson".into(),
            version: "0.0".into(),
        })
    }
}

```


Could you search the internet to see what the right method to handle this would be and try to suggest a solution to this problem?


Before you finish your task run ` RUSTFLAGS="-A warnings" cargo check --message-format=short` (Some optimisations to weed out a bunch of unneded tokens) to make sure you havent made any mistakes. Also try to avoid modifying any code that isnt absolutely essential to implement your feature.
