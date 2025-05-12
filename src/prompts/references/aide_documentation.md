# General Setup:
Take this framework
```rs
use axum::{response::IntoResponse, routing::post, Json, Router};
use serde::Deserialize;

#[derive(Deserialize)]
struct User {
    name: String,
}

async fn hello_user(Json(user): Json<User>) -> impl IntoResponse {
    format!("hello {}", user.name)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/hello", post(hello_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
```
and do the following to add api support:

```rs
// Replace some of the `axum::` types with `aide::axum::` ones.
use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
};
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::Deserialize;

// We'll need to derive `JsonSchema` for
// all types that appear in the api documentation.
#[derive(Deserialize, JsonSchema)]
struct User {
    name: String,
}

async fn hello_user(Json(user): Json<User>) -> impl IntoApiResponse {
    format!("hello {}", user.name)
}

// Note that this clones the document on each request.
// To be more efficient, we could wrap it into an Arc,
// or even store it as a serialized string.
async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[tokio::main]
async fn main() {
    let app = ApiRouter::new()
        // Change `route` to `api_route` for the route
        // we'd like to expose in the documentation.
        .api_route("/hello", post(hello_user))
        // We'll serve our generated document here.
        .route("/api.json", get(serve_api));

    let mut api = OpenApi {
        info: Info {
            description: Some("an example API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();

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
}
```
We can add more documentation like so:
```rs
// ...
.api_route(
    "/hello",
    post_with(hello_user, |o| {
        o.id("helloUser")
            .description("says hello to the given user")
            .response_with::<200, String, _>(|res| {
                res.description("a simple message saying hello to the user")
                    .example(String::from("hello Tom"))
            })
    }),
)
// ...
```
# Documentation for setting up swagger

```rs
// Replace some of the `axum::` types with `aide::axum::` ones.
use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
    swagger::Swagger,
};
use axum::{Extension, Json};
use schemars::JsonSchema;
use serde::Deserialize;

// We'll need to derive `JsonSchema` for
// all types that appear in the api documentation.
#[derive(Deserialize, JsonSchema)]
struct User {
    name: String,
}

async fn hello_user(Json(user): Json<User>) -> impl IntoApiResponse {
    format!("hello {}", user.name)
}

// Note that this clones the document on each request.
// To be more efficient, we could wrap it into an Arc,
// or even store it as a serialized string.
async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[tokio::main]
async fn main() {
    let app = ApiRouter::new()
        // generate swagger-ui using the openapi spec route
        .route("/swagger", Swagger::new("/api.json").axum_route())
        // Change `route` to `api_route` for the route
        // we'd like to expose in the documentation.
        .api_route("/hello", post(hello_user))
        // We'll serve our generated document here.
        .route("/api.json", get(serve_api));

    let mut api = OpenApi {
        info: Info {
            description: Some("an example API".to_string()),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

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
}
```
