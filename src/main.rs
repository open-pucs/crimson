use std::io;
use std::net::Ipv4Addr;

use tokio::net::TcpListener;
use utoipa::OpenApi;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;
use utoipa_swagger_ui::SwaggerUi;

mod docs;

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
