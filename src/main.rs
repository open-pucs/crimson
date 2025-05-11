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
    )
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
    "ok"
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
    use serde::Serialize;
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    /// This is the customer
    #[derive(ToSchema, Serialize)]
    struct Customer {
        name: String,
    }

    /// expose the Customer OpenAPI to parent module
    pub fn router() -> OpenApiRouter {
        OpenApiRouter::new().routes(routes!(get_customer))
    }

    /// Get customer
    ///
    /// Just return a static Customer object
    #[utoipa::path(get, path = "", responses((status = OK, body = Customer)), tag = super::DOCS_TAG)]
    async fn get_customer() -> Json<Customer> {
        Json(Customer {
            name: String::from("Bill Book"),
        })
    }
}

mod admin {
    use axum::Json;
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;
    use utoipa_axum::router::OpenApiRouter;
    use utoipa_axum::routes;

    /// This is the order
    #[derive(ToSchema, Serialize)]
    struct Order {
        id: i32,
        name: String,
    }

    #[derive(ToSchema, Deserialize, Serialize)]
    struct OrderRequest {
        name: String,
    }

    /// expose the Order OpenAPI to parent module
    pub fn router() -> OpenApiRouter {
        OpenApiRouter::new().routes(routes!(get_order, create_order))
    }

    /// Get static order object
    #[utoipa::path(get, path = "", responses((status = OK, body = Order)), tag = super::ADMIN_TAG)]
    async fn get_order() -> Json<Order> {
        Json(Order {
            id: 100,
            name: String::from("Bill Book"),
        })
    }

    /// Create an order.
    ///
    /// Create an order by basically passing through the name of the request with static id.
    #[utoipa::path(post, path = "", responses((status = OK, body = Order)), tag = super::ADMIN_TAG)]
    async fn create_order(Json(order): Json<OrderRequest>) -> Json<Order> {
        Json(Order {
            id: 120,
            name: order.name,
        })
    }
}
