use std::io;
use std::net::Ipv4Addr;

use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

const DOCS_TAG: &str = "docs";
const ADMIN_TAG: &str = "admin";

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = DOCS_TAG, description = "Document API endpoints"),
        (name = ADMIN_TAG, description = "Admin API endpoints")
    ),
    paths(health)
)]
struct ApiDoc;

/// Get health of the API.
#[utoipa::path(
    method(get, head),
    path = "/api/health",
    responses(
        (status = OK, description = "Success", body = str, content_type = "text/plain")
    )
)]
async fn health() -> &'static str {
    "Service is Healthy"
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    let (router, api) = OpenApiRouter::with_openapi(ApiDoc::openapi())
        .routes(routes!(health))
        .nest("/api/docs", docs::router())
        .nest("/api/admin", admin::router())
        .split_for_parts();

    let router = router.merge(SwaggerUi::new("/swagger-ui").url("/apidoc/openapi.json", api));

    let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 8080)).await?;
    axum::serve(listener, router).await
}

mod docs {
    use axum::Json;
    use axum::extract::Multipart;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;
    use uuid::Uuid;
    #[derive(ToSchema, Serialize, Deserialize)]
    enum ProcessingStage {
        Completed,
        Waiting,
        Errored,
        Processing,
    }

    // PDF Processing Data.
    #[derive(ToSchema, Deserialize, Serialize)]
    struct PdfProcessingInfo {
        id: Uuid, // Figure out what a proper type for a uuid in rust is.
        process_stage: ProcessingStage,
    }

    /// PDF upload data
    #[derive(Deserialize, ToSchema)]
    pub struct PdfUpload {
        #[schema(format = Binary, content_media_type = "application/pdf")]
        pdf: String,
    }

    /// Ingest a PDF file via multipart/form-data
    #[utoipa::path(
    post,
    path = "/api/ingest",
    request_body(content = PdfUpload, content_type = "multipart/form-data"),
    responses(
        (status = 200, description = "PDF successfully ingested", body = String)
    ),
     tag = super::DOCS_TAG
)]
    async fn pdf_ingest(mut multipart: Multipart) -> Json<PdfProcessingInfo> {
        while let Some(field) = multipart.next_field().await.unwrap() {
            if let Some(name) = field.name() {
                if name == "pdf" {
                    let _bytes = field.bytes().await.expect("should be bytes for pdf field");
                    break;
                }
            }
        }
        Json(PdfProcessingInfo {
            id: Uuid::new_v4(),
            process_stage: ProcessingStage::Waiting,
        })
    }

    /// expose the Customer OpenAPI to parent module
    pub fn router() -> OpenApiRouter {
        OpenApiRouter::new().routes(routes!(pdf_ingest))
    }
}

mod admin {
    use axum::Json;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    #[derive(ToSchema, Deserialize, Serialize)]
    struct ServerInfo {
        name: String,
        version: String,
    }

    /// expose the Order OpenAPI to parent module
    pub fn router() -> OpenApiRouter {
        OpenApiRouter::new().routes(routes!(get_server_info))
    }

    /// Get static order object
    #[utoipa::path(get, path = "/info", responses((status = OK, body = ServerInfo)), tag = super::ADMIN_TAG)]
    async fn get_server_info() -> Json<ServerInfo> {
        Json(ServerInfo {
            name: "Crimson".into(),
            version: "0.0".into(),
        })
    }
}
