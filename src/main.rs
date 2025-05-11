use axum::{extract::Multipart, routing::{get, post}, Router, serve};
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use std::net::SocketAddr;

/// Schemas for request bodies
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct HelloForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct PdfUpload {
    #[schema(format = Binary, content_media_type = "application/pdf")]
    pdf: String,
}

/// OpenAPI documentation
#[derive(OpenApi)]
#[openapi(
    paths(hello_form, pdf_ingest, health_check),
    components(schemas(HelloForm, PdfUpload)),
    tags(
        (name = "hello", description = "Hello form endpoint"),
        (name = "ingest", description = "PDF ingestion endpoint"),
        (name = "health", description = "Health check endpoint")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate OpenAPI spec
    let openapi = ApiDoc::openapi();

    // Build application router
    let app = Router::new()
        .route("/hello", post(hello_form))
        .route("/ingest", post(pdf_ingest))
        .route("/health", get(health_check))
        .merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api/openapi.json", openapi)
        );

    // Bind and serve
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    println!("Listening on {}", addr);
    // Bind to address
    let listener = tokio::net::TcpListener::bind(addr).await?;
    // Serve the application
    serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[utoipa::path(
    post,
    path = "/hello",
    request_body(content = HelloForm, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "A greeting with file info", body = String)
    )
)]
async fn hello_form(mut multipart: Multipart) -> String {
    let mut name: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut size: usize = 0;
    let mut file_name: Option<String> = None;

    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(field_name) = field.name() {
            match field_name {
                "name" => {
                    name = Some(field.text().await.expect("should be text for name field"));
                }
                "file" => {
                    file_name = field.file_name().map(ToString::to_string);
                    content_type = field.content_type().map(ToString::to_string);
                    let bytes = field.bytes().await.expect("should be bytes for file field");
                    size = bytes.len();
                }
                _ => (),
            }
        }
    }
    format!(
        "name: {}, content_type: {}, size: {}, file_name: {}",
        name.unwrap_or_default(),
        content_type.unwrap_or_default(),
        size,
        file_name.unwrap_or_default()
    )
}

#[utoipa::path(
    post,
    path = "/ingest",
    request_body(content = PdfUpload, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "PDF successfully ingested", body = String)
    )
)]
async fn pdf_ingest(mut multipart: Multipart) -> String {
    // Iterate through the multipart fields, look for `pdf`
    while let Some(field) = multipart.next_field().await.unwrap() {
        if let Some(name) = field.name() {
            if name == "pdf" {
                let _bytes = field.bytes().await.expect("should be bytes for pdf field");
                break;
            }
        }
    }
    String::from("Successfully ingested")
}

#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is Healthy", body = String)
    )
)]
async fn health_check() -> String {
    "Service is Healthy".into()
}