use std::net::SocketAddr;

use axum::{
    extract::Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Deserialize, Serialize)]
pub struct PathParam {
    pub user_id: String,
}

// curl 127.0.0.1:8080/api/users/123
pub async fn user_detail(Path(params): Path<PathParam>) -> impl IntoResponse {
    println!("{:?}", params);
    (StatusCode::OK, Json(params)).into_response()
}

#[derive(Debug, Serialize, Deserialize)]
struct Hello {
    message: String,
}

// curl "127.0.0.1:8080/echo" -H "Content-Type: application/json" -d {message:"hello world"}
// curl -H "Content-Type: application/json" -d '{"message":"echo msg"}' -X POST 127.0.0.1:8080/echo
async fn echo(item: Json<Hello>) -> impl IntoResponse {
    println!("{:?}", item);
    Json(json!({ "data": 42 }))
}

// `Json` gives a content-type of `application/json` and works with any type
// that implements `serde::Serialize`
async fn json() -> Json<Value> {
    Json(json!({ "data": 42 }))
}

// curl "127.0.0.1:8080/"
async fn index() -> String {
    String::from("hello axum index")
}
// curl "127.0.0.1:8080/foo"
async fn get_foo() -> String {
    String::from("get:foo")
}
async fn post_foo() -> String {
    String::from("post:foo")
}
// curl "127.0.0.1:8080/foo/bar"
async fn foo_bar() -> String {
    String::from("foo:bar")
}

// a static string的处理器
async fn root() -> &'static str {
    "Hello, World!"
}

// curl -H "Content-Type: application/json" -d '{"username":"someName"}' -X POST http://127.0.0.1:8080/users
async fn create_user(
    // 这个参数告诉 axum 把请求体是Json格式的，代表CrateUser类型
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // 响应内容为Json格式，状态码是201
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // our router
    let app = Router::new()
        .route("/", get(root))
        .route("/index", get(index))
        .route("/json", get(json))
        .route("/echo", post(echo))
        .route("/foo", get(get_foo).post(post_foo))
        .route("/foo/bar", get(foo_bar))
        .route("/api/users/:user_id", get(user_detail))
        .route("/users", post(create_user));

    // run it with hyper on localhost:8080
    // axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
