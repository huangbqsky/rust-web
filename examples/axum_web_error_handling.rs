use axum::{
    body::Body,
    error_handling::HandleError,
    http::{Response, StatusCode},
    Router,
};

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

#[tokio::main]
async fn main() {
    // 这个 Service 可能出现任何错误
    let some_fallible_service = tower::service_fn(|_req| async {
        thing_that_might_fail().await?;
        Ok::<_, anyhow::Error>(Response::new(Body::empty()))
    });

    let app = Router::new().route_service(
        "/",
        // Service 适配器通过将错误转换为响应来处理错误。
        HandleError::new(some_fallible_service, handle_anyhow_error),
    );

    let addr = "127.0.0.1:3000";
    println!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
