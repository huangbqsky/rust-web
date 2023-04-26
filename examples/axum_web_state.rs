use axum::{
    extract::{Path, State},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use std::sync::Arc;

// 共享状态结构体
struct AppState {
    // ...
}

// 方法1: 使用 State 状态提取器
async fn handler_as_state_extractor(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // ...
    String::from("State extract shared_state")
}

// 方法2: 使用 Extension 请求扩展提取器
// 这种方法的缺点是，如果您尝试提取一个不存在的扩展，可能是因为您忘记添加中间件，
// 或者因为您提取了错误的类型，那么您将得到运行时错误(特别是500 Internal Server Error 响应)。
async fn handler_as_extension_extractor(Extension(state): Extension<Arc<AppState>>) -> impl IntoResponse {
    // ...
    String::from("Extension extract shared_state")
}

// 方法3: 使用闭包捕获（closure captures）直接传递给处理器
async fn get_user(Path(user_id): Path<String>, state: Arc<AppState>) -> impl IntoResponse {
    // ...
    String::from("closure captures shared_state")
}

#[tokio::main]
async fn main() {
    // 处理器共享状态（Sharing state with handlers），使用一种方式即可
    let shared_state = Arc::new(AppState { /* ... */ });
    let shared_state_extension = Arc::clone(&shared_state);
    let shared_state_closure = Arc::clone(&shared_state);

    let app = Router::new()
        .route("/state", get(handler_as_state_extractor)) // 1.使用State提取器
        .with_state(shared_state)
        .route("/extension", get(handler_as_extension_extractor)) // 2.使用Extension提取器
        .layer(Extension(shared_state_extension))
        .route(
            "/users/:id",
            get({
                move |path| get_user(path, shared_state_closure)  // 3.使用闭包捕获直接传递给处理器
            }),
        );

    let addr = "127.0.0.1:3000";
    println!("listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
