
use axum::{Router, routing::get, extract::Json, response::IntoResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Hello {
    message: String,
}

// curl "127.0.0.1:8080/hello" -H "Content-Type: application/json" -d {"Hello":{message:"hello world"}}
async fn hello(item: Json<Hello>) ->impl IntoResponse {
    println!("{}", item.message);
    item 
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
      .route("/hello", get(hello))
      .route("/foo", get(get_foo).post(post_foo))
      .route("/foo/bar", get(foo_bar));

    // run it with hyper on localhost:8080
    axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
      .serve(app.into_make_service())
      .await
      .unwrap();
}