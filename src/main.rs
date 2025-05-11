use axum::{
    Json, Router, extract::Multipart, http::StatusCode, response::IntoResponse, routing::post,
};
use serde::Deserialize;
use utoipa::OpenApi;
use utoipa::ToSchema;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Deserialize, ToSchema)]
pub struct UploadForm {
    #[schema(format = Binary, description = "PDF file to ingest")]
    file: Vec<u8>,
}

/// Upload endpoint that ingests a PDF
#[utoipa::path(
    post,
    path = "/upload",
    request_body(content = UploadForm, content_type = "multipart/form-data"),
    responses((status = 200, description = "Successfully ingested"))
)]
async fn upload(mut multipart: Multipart) -> Result<impl IntoResponse, (StatusCode, String)> {
    while let Some(field) = multipart.next_field().await.map_err(internal_error)? {
        let name = field.name().unwrap_or("");
        if name == "file" {
            let data = field.bytes().await.map_err(internal_error)?;
            // TODO: process PDF bytes (e.g., send to processing pipeline)
            println!("Received PDF with {} bytes", data.len());
            return Ok((StatusCode::OK, "Successfully ingested"));
        }
    }
    Err((
        StatusCode::BAD_REQUEST,
        "No file field in multipart".to_string(),
    ))
}

fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}

#[derive(OpenApi)]
#[openapi(
    paths(upload),
    components(schemas(UploadForm)),
    info(
        title = "Crimson PDF Processor",
        version = "0.1.0",
        description = "API for high throughput PDF ingestion"
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        .route("/upload", post(upload))
        .merge(SwaggerUi::new("/docs").url("/api-doc/openapi.json", ApiDoc::openapi()));

    // run it
    let addr = "0.0.0.0:8080";
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
