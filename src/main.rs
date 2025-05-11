use axum::extract::Multipart;
use serde::Deserialize;
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let (router, api) = OpenApiRouter::new()
        .routes(routes!(pdf_ingest, hello_form, health_check))
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/api/openapi.json", api));

    let app = router.into_make_service();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await
}

/// Just a schema for axum native multipart
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct HelloForm {
    name: String,
    #[schema(format = Binary, content_media_type = "application/octet-stream")]
    file: String,
}

/// Schema for PDF ingestion multipart upload
#[derive(Deserialize, ToSchema)]
#[allow(unused)]
struct PdfUpload {
    #[schema(format = Binary, content_media_type = "application/pdf")]
    pdf: String,
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
        let field_name = field.name();

        match &field_name {
            Some("name") => {
                name = Some(field.text().await.expect("should be text for name field"));
            }
            Some("file") => {
                file_name = field.file_name().map(ToString::to_string);
                content_type = field.content_type().map(ToString::to_string);
                let bytes = field.bytes().await.expect("should be bytes for file field");
                size = bytes.len();
            }
            _ => (),
        };
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
                // Here you could process the PDF bytes, e.g., save or analyze
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
    request_body(),
    responses(
        (status = 200, description = "Service is Healthy", body = String)
    )
)]
async fn health_check() -> String {
    "Service is Healthy".into()
}

