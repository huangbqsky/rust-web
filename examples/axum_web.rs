
use axum::{Router, routing::{get, post}, extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Serialize, Deserialize)]
struct Hello {
    message: String,
}

// curl "127.0.0.1:8080/hello" -H "Content-Type: application/json" -d {"message":"hello world"}
async fn echo(item: Json<Hello>) ->impl IntoResponse {
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

#[tokio::main]
async fn main() {
    // let app = Router::new().route("/", post(index));

    // let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    // axum::Server::bind(&addr)
    //     .serve(app.into_make_service())
    //     .await
    //     .unwrap();

    // our router
    let app = Router::new()
      .route("/", get(index))
      .route("/json", get(json))
      .route("/echo", post(echo))
      .route("/foo", get(get_foo).post(post_foo))
      .route("/foo/bar", get(foo_bar));

    // run it with hyper on localhost:8080
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
      .serve(app.into_make_service())
      .await
      .unwrap();
}