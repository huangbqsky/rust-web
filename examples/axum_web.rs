use std::{net::SocketAddr, collections::HashMap};

use axum::{
    extract::{Json, Query, TypedHeader},
    extract::Path,
    http::{StatusCode, HeaderMap, HeaderValue},
    response::IntoResponse,
    routing::{get, post},
    Router, headers,
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

// json提交
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
    String::from("hello axum")
}

// json提交
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

// eg: /user/30，将解析出id=30
async fn user_info(Path(id): Path<i32>) -> String {
    format!("user id:{}", id)
}
 
// eg: /user2/30，将解析出id=30
async fn user_info_2(id: Path<i32>) -> String {
    format!("user id:{}", id.0)
}
 
// eg: /person/123/30，将解析出id=123, age=30
async fn person(Path((id, age)): Path<(i32, i32)>) -> String {
    format!("id:{},age:{}", id, age)
}
 
#[derive(Deserialize)]
struct SomeRequest2 {
    a: Option<String>,
    b: Option<i32>,
    c: Option<String>,
    d: Option<u32>,
}
 
#[derive(Deserialize)]
struct SomeRequest {
    a: String,
    b: i32,
    c: String,
    d: u32,
}
 
// eg: path_req/a1/b1/c1/d1
async fn path_req(Path(req): Path<SomeRequest>) -> String {
    format!("a:{},b:{},c:{},d:{}", req.a, req.b, req.c, req.d)
}
 
//eg: query_req/?a=test&b=2&c=abc&d=80
async fn query_req(Query(args): Query<SomeRequest>) -> String {
    format!("a:{},b:{},c:{},d:{}", args.a, args.b, args.c, args.d)
}
 
//eg: query_req2?a=abc&c=中华人民共和国&d=123
async fn query_req2(Query(args): Query<SomeRequest2>) -> String {
    format!(
        "a:{},b:{},c:{},d:{}",
        args.a.unwrap_or_default(),
        args.b.unwrap_or(-1), //b缺省值指定为-1
        args.c.unwrap_or_default(),
        args.d.unwrap_or_default()
    )
}

/**
 * 获取请求参数
 * http://127.0.0.1:8080/query?a=1&b=1.0&c=xxx
 */
async fn query(Query(params): Query<HashMap<String, String>>) -> String {
    for (key, value) in &params {
        println!("key:{},value:{}", key, value);
    }
    format!("{:?}", params)
}

/**
 * 获取所有请求头
 * http://127.0.0.1:8080/header
 */
async fn get_all_header(headers: HeaderMap) -> String {
    for (key, value) in &headers {
        println!("key:{:?} , value:{:?}", key, value);
    }
    format!("{:?}", headers)
}
/**
 * 获取http headers中的user_agent头
 * http://127.0.0.1:8080/user_agent
 */
async fn get_user_agent_header(TypedHeader(user_agent): TypedHeader<headers::UserAgent>) -> String {
    user_agent.to_string()
}

/**
 * 设置cookie并跳转到新页面
 * http://127.0.0.1:8080/get_cookie
 */
async fn set_cookie_and_redirect(mut headers: HeaderMap) -> (StatusCode, HeaderMap, ()) {
    //设置cookie，blog_url为cookie的key
    headers.insert(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str("request_url=https://github.com/tokio-rs/axum/").unwrap(),
    );
 
    //重设LOCATION，跳到新页面
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str("/get_cookie").unwrap(),
    );
    //302重定向
    (StatusCode::FOUND, headers, ())
}
 
/**
 * 读取cookie
 * http://127.0.0.1:8080/set_cookie
 */
async fn get_cookie(headers: HeaderMap) -> (StatusCode, String) {
    //读取cookie，并转成字符串
    let cookies = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string())
        .unwrap_or("".to_string());
 
    //cookie空判断
    if cookies.is_empty() {
        println!("cookie is empty!");
        return (StatusCode::OK, "cookie is empty".to_string());
    }
 
    //将cookie拆成列表
    let cookies: Vec<&str> = cookies.split(';').collect();
    println!("{:?}", cookies);
    for cookie in &cookies {
        //将内容拆分成k=v的格式
        let cookie_pair: Vec<&str> = cookie.split('=').collect();
        if cookie_pair.len() == 2 {
            let cookie_name = cookie_pair[0].trim();
            let cookie_value = cookie_pair[1].trim();
            println!("{:?}", cookie_pair);
            //判断其中是否有刚才设置的blog_url
            if cookie_name == "request_url" && !cookie_value.is_empty() {
                println!("found:{}", cookie_value);
                return (StatusCode::OK, cookie_value.to_string());
            }
        }
    }
    return (StatusCode::OK, "empty".to_string());
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // our router
    let app = Router::new()
        .route("/", get(index))
        .route("/json", get(json))
        .route("/echo", post(echo))
        .route("/api/users/:user_id", get(user_detail))
        .route("/users", post(create_user))
        .route("/user/:id", get(user_info))
        .route("/user2/:id", get(user_info_2))
        .route("/person/:id/:age", get(person))
        .route("/path_req/:a/:b/:c/:d", get(path_req))
        .route("/query_req", get(query_req))
        .route("/query_req2", get(query_req2))
        .route("/query", get(query)) //获取请求参数
        .route("/header", get(get_all_header)) //获取所有请求头
        .route("/user_agent", get(get_user_agent_header)) // 获取user_agent头
        .route("/set_cookie", get(set_cookie_and_redirect)) // 设置cookie并跳转到新页面
        .route("/get_cookie", get(get_cookie)); // 读取cookie

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
