use std::time::Duration;

use axum::{
    body::Body,
    error_handling::{HandleError, HandleErrorLayer},
    http::{Method, Response, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    BoxError, Router,
};
use tower::ServiceBuilder;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .merge(router_fallible_service()) // 模拟使用 Service的错误处理
        .merge(router_fallible_middleware()) // 模拟使用中间件的错误处理
        .merge(router_fallible_extractor());  // 模拟使用提取器的错误处理  

    let addr = "127.0.0.1:3000";
    println!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// 错误处理方式1: 模拟使用 Service的错误处理
fn router_fallible_service() -> Router {
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

// 处理器：模拟超时
async fn handler_timeout() -> impl IntoResponse {
    println!("sleep 3 seconds");
    tokio::time::sleep(Duration::from_secs(3)).await; // 休眠3秒，模拟超时
    format!("Hello Error Handling !!!")
}

// 错误处理方式2 : 用中间件处理错误的路由
fn router_fallible_middleware() -> Router {
    Router::new()
        .route("/fallible_middleware", get(handler_timeout))
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handler_timeout_error))
                .timeout(Duration::from_secs(1)),
        )
}

// 用中间件处理错误
async fn handler_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            "Request time too long， Timeout！！！".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unhandled internal error: {}", err),
        )
    }
}

// 错误处理方式3: 用运行时提取器处理错误的路由
fn router_fallible_extractor() -> Router {
    Router::new()
        .route("/fallible_extractor", get(handler_timeout))
        .layer(
            ServiceBuilder::new()
                // `timeout` will produce an error if the handler takes
                // too long so we must handle those
                .layer(HandleErrorLayer::new(handler_timeout_fallible_extractor))
                .timeout(Duration::from_secs(1)),
        )
}

// 用运行时提取器 Extractor中间件处理错误
async fn handler_timeout_fallible_extractor(
    // `Method` and `Uri` are extractors so they can be used here
    method: Method,
    uri: Uri,
    // the last argument must be the error itself
    err: BoxError,
) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("`{} {}` failed with {}", method, uri, err),
    )
}