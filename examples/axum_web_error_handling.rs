use std::{thread::sleep, time::Duration};

use axum::{
    body::Body,
    error_handling::{HandleError, HandleErrorLayer},
    http::{Method, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    BoxError, Router,
};
use tower::ServiceBuilder;

// 业务处理逻辑，可能出现失败而抛出 Error
async fn thing_that_might_fail() -> Result<(), anyhow::Error> {
    // 模拟一个错误
    anyhow::bail!("thing_that_might_fail")
}

// 把错误转化为 IntoResponse
async fn handle_anyhow_error(err: anyhow::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Something went wrong: {}", err),
    )
}

// https://docs.rs/axum/latest/axum/error_handling/index.html
#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(router_fallible_middleware())
        .merge(router_fallible_extractor())
        .merge(router_error());

    let addr = "127.0.0.1:3000";
    println!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn router_error() -> Router {
    // 这个 Service 可能出现任何错误
    let some_fallible_service = tower::service_fn(|_req| async {
        thing_that_might_fail().await?;
        Ok::<_, anyhow::Error>(Response::new(Body::empty()))
    });

    Router::new().route_service(
        "/",
        // Service 适配器通过将错误转换为响应来处理错误。
        HandleError::new(some_fallible_service, handle_anyhow_error),
    )
}

async fn handle() -> impl IntoResponse {
    sleep(Duration::from_secs(5));
    println!("handle_timeout_error");
    format!("handle_timeout_error")
}

// 用中间件处理错误
fn router_fallible_middleware() -> Router {
    Router::new()
        .route("/fallible_middleware", get(handle))
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(3)),
        )
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request took too long".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

// 用运行时提取器中间件处理错误
fn router_fallible_extractor() -> Router {
    Router::new()
        .route("/fallible_extractor", get(handle))
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handle_timeout_fallible_extractor))
                .timeout(Duration::from_secs(3)),
        )
}

async fn handle_timeout_fallible_extractor(
    // `Method` and `Uri` are extractors so they can be used here
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    let msg = format!("`{} {}` failed with {}", method, uri, err);
    println!("{}", msg);
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        msg,
    )
}
