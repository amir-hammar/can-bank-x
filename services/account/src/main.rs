use axum::{http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Deserialize, ToSchema)]
struct CreateAccountRequest {
    client_id: String,
    initial_balance: f64,
}

#[derive(Serialize, ToSchema)]
struct CreateAccountResponse {
    account_id: String,
    status: String,
}

#[derive(Serialize, ToSchema)]
struct ErrorResponse {
    code: String,
    message: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/create-account",
    request_body = CreateAccountRequest,
    responses(
        (status = 201, description = "Account created", body = CreateAccountResponse),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 500, description = "Server error", body = ErrorResponse)
    ),
    tag = "account"
)]
async fn create_account(Json(payload): Json<CreateAccountRequest>) -> Result<(StatusCode, Json<CreateAccountResponse>), (StatusCode, Json<ErrorResponse>)> {
    if payload.initial_balance < 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                code: "INVALID_INITIAL_BALANCE".to_string(),
                message: "initial_balance must be >= 0".to_string(),
            }),
        ));
    }

    let response = CreateAccountResponse {
        account_id: format!("acc_{}", payload.client_id),
        status: "created".to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[derive(OpenApi)]
#[openapi(
    paths(create_account),
    components(schemas(CreateAccountRequest, CreateAccountResponse, ErrorResponse)),
    tags((name = "account", description = "Account operations"))
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let openapi = ApiDoc::openapi();

    let app = Router::new()
        .route("/api/v1/create-account", post(create_account))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", openapi));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080")
        .await
        .expect("failed to bind to 0.0.0.0:8080");

    axum::serve(listener, app).await.expect("server error");
}
