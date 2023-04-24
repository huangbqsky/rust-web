use std::convert::Infallible;
use std::time::Duration;

use salvo::cache::{Cache, MemoryStore, RequestIssuer};
use salvo::macros::Extractible;
use salvo::sse::SseEvent;
use salvo::writer::Text;
use salvo::{prelude::*, Catcher};
use time::OffsetDateTime;

use futures::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use serde::{Deserialize, Serialize};

#[handler]
async fn show(req: &mut Request, res: &mut Response) {
    let content = format!(
        r#"<!DOCTYPE html>
    <html>
        <head>
            <title>Parse data</title>
        </head>
        <body>
            <h1>Hello, fill your profile</h1>
            <div id="result"></div>
            <form id="form" method="post">
                <label>First Name:</label><input type="text" name="first_name" />
                <label>Last Name:</label><input type="text" name="last_name" />
                <legend>What is Your Favorite Pet?</legend>      
                <input type="checkbox" name="lovers" value="Cats">Cats<br>      
                <input type="checkbox" name="lovers" value="Dogs">Dogs<br>      
                <input type="checkbox" name="lovers" value="Birds">Birds<br>    
                <input type="submit" value="Submit" />
            </form>
            <script> 
            let form = document.getElementById("form");
            form.addEventListener("submit", async (e) => {{
                e.preventDefault();
                let response = await fetch('/{}?username=jobs', {{
                    method: 'POST',
                    headers: {{
                        'Content-Type': 'application/json',
                    }},
                    body: JSON.stringify({{
                        first_name: form.querySelector("input[name='first_name']").value,
                        last_name: form.querySelector("input[name='last_name']").value,
                        lovers: Array.from(form.querySelectorAll("input[name='lovers']:checked")).map(el => el.value),
                    }}),
                }});
                let text = await response.text();
                document.getElementById("result").innerHTML = text;
            }});
            </script>
        </body>
    </html>
    "#,
        req.params().get("id").unwrap()
    );
    res.render(Text::Html(content));
}

#[handler]
async fn edit<'a>(good_man: GoodMan<'a>, res: &mut Response) {
    res.render(Json(good_man));
}

#[derive(Serialize, Deserialize, Extractible, Debug)]
#[extract(default_source(from = "body", format = "json"))]
struct GoodMan<'a> {
    #[extract(source(from = "param"))]
    id: i64,
    #[extract(source(from = "query"))]
    username: &'a str,
    first_name: String,
    last_name: String,
    lovers: Vec<String>,
    #[extract(source(from = "request"))]
    nested: Nested<'a>,
}

#[derive(Serialize, Deserialize, Extractible, Debug)]
#[extract(default_source(from = "body", format = "json"))]
struct Nested<'a> {
    #[extract(source(from = "param"))]
    id: i64,
    #[extract(source(from = "query"))]
    username: &'a str,
    first_name: String,
    last_name: String,
    #[extract(rename = "lovers")]
    #[serde(default)]
    pets: Vec<String>,
}

#[handler]
async fn hello(req: &mut Request) -> String {
    let query_id = req.query::<String>("id").unwrap();
    let param_id = req.params().get("id").cloned().unwrap_or_default();
    println!("querys id= {:?}, params id = {:?}", query_id, param_id);

    format!("{:?}", req)
}

#[handler]
async fn index() -> &'static str {
    "Hello world"
}

#[derive(Serialize, Debug)]
struct User {
    name: String,
}

// 在 Handler 中, Response 会被作为参数传入
#[handler]
async fn show_resp(res: &mut Response, ctrl: &mut FlowCtrl) {
    ctrl.skip_rest();
    // 写入纯文本数据
    // res.render("Hello world! Response and FlowCtrl example");

    // 写入 HTML
    // res.render(Text::Html("<html><body>Hello Salvo! Response and FlowCtrl example</body></html>"));

    // 写入 JSON 序列化数据
    // let user = User{name: "Johee".to_string()};
    // res.render(Json(user));

    // 写入错误信息代码
    // res.set_status_code(StatusCode::BAD_REQUEST);
    // 写入详细错误信息
    res.set_status_error(
        StatusError::internal_server_error().with_summary("error when serialize object to json"),
    );
}

#[derive(Default, Debug)]
struct Config {
    id: i32,
}

// Depot 是用于保存一次请求中涉及到的临时数据. 中间件可以将自己处理的临时数据放入 Depot, 供后续程序使用.
// 比如说, 我们可以在登录的中间件中设置 current_user, 然后在后续的中间件或者 Handler 中读取当前用户信息.
// 通过 insert 和 get 设置和取出k-v键值对数据
// 通过 inject 和 obtain 设置和取出非键值对数据
#[handler]
async fn set_user(depot: &mut Depot) {
    // 插入键值对数据到 Depot中
    depot.insert("current_user", "Elon Musk");

    // 不需要关系具体 key 的数据保存
    depot.inject(Config::default());
}
#[handler]
async fn hoop(depot: &mut Depot) -> String {
    // 取出数据非键值对数据
    let config = depot.obtain::<Config>().unwrap();

    // 需要注意的是, 这里的类型必须是 &str, 而不是 String, 因为当初存入的数据类型为 &str.
    let user = depot.get::<&str>("current_user").copied().unwrap();
    format!(
        "Hey {}, I love your money and girls!， config id: {:?}",
        user, config.id
    )
}

// 自定义错误类型
struct CustomError;
#[async_trait]
impl Writer for CustomError {
    async fn write(mut self, _req: &mut Request, _depot: &mut Depot, res: &mut Response) {
        res.render("custom error");
        res.set_status_error(StatusError::internal_server_error());
    }
}

#[handler]
async fn handle_custom() -> Result<(), CustomError> {
    Err(CustomError)
}

// Catcher 是用于处理页面返回 HTTP 状态码为错误的情况下, 如何显示页面的抽象.
struct Handle404;
impl Catcher for Handle404 {
    fn catch(&self, _req: &Request, _depot: &Depot, res: &mut Response) -> bool {
        if let Some(StatusCode::NOT_FOUND) = res.status_code() {
            res.render("Custom 404 Error Page");
            true
        } else {
            false
        }
    }
}

#[handler]
async fn home() -> Text<&'static str> {
    Text::Html(
        r#"
    <!DOCTYPE html>
    <html>
        <head>
            <title>Cache Example</title>
        </head>
        <body>
            <h2>Cache Example</h2>
            <p>
                This examples shows how to use cache middleware. 
            </p>
            <p>
                <a href="/short" target="_blank">Cache 5 seconds</a>
            </p>
            <p>
                <a href="/long" target="_blank">Cache 1 minute</a>
            </p>
        </body>
    </html>
    "#,
    )
}

#[handler]
async fn short() -> String {
    format!(
        "Hello World, my birth time is {}",
        OffsetDateTime::now_utc()
    )
}
#[handler]
async fn long() -> String {
    format!(
        "Hello World, my birth time is {}",
        OffsetDateTime::now_utc()
    )
}

// create server-sent event
fn sse_counter(counter: u64) -> Result<SseEvent, Infallible> {
    Ok(SseEvent::default().data(counter.to_string()))
}

// sse 测试用例: 每3秒向客户端发送一次消息，每1秒发送一次保活
#[handler]
async fn handle_tick(res: &mut Response) {
    let event_stream = {
        let mut counter: u64 = 0;
        let interval = interval(Duration::from_secs(3));
        let stream = IntervalStream::new(interval);
        // 每3秒，向浏览器发1次消息
        stream.map(move |_| {
            counter += 1;
            // create server-sent event
            sse_counter(counter)
        })
    };

    // sse::streaming(res, event_stream).ok();

    //每隔1秒发1次保活(可以理解成心跳包)
    SseKeepAlive::new(event_stream)
        .with_comment("keep-alive-text")
        .with_interval(Duration::from_secs(1))
        .streaming(res)
        .unwrap();
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let addr = "127.0.0.1:7878";
    println!("listening on {}", addr);
    tracing::info!("Listening on http://127.0.0.1:7878");

    // Cache 中间件可以对 Response 中的 StatusCode, Headers, Body 提供缓存功能
    let short_cache = Cache::new(
        MemoryStore::builder()
            .time_to_live(Duration::from_secs(5))
            .build(),
        RequestIssuer::default(),
    );
    let long_cache = Cache::new(
        MemoryStore::builder()
            .time_to_live(Duration::from_secs(60))
            .build(),
        RequestIssuer::default(),
    );

    // 路由
    let router = Router::new()
        .get(home)
        .push(Router::with_path("short").hoop(short_cache).get(short)) // http://127.0.0.1:7878/short  中间件 short_cache
        .push(Router::with_path("long").hoop(long_cache).get(long)) // http://127.0.0.1:7878/long  中间件 long_cache
        .push(Router::with_path("hoop").hoop(set_user).get(hoop)) // http://127.0.0.1:7878/hoop  中间件 set_user
        .push(Router::with_path("ticks").get(handle_tick)) // http://127.0.0.1:7878/ticks  向客户端发送消息
        .push(Router::with_path("index").get(index)) // http://127.0.0.1:7878/index
        .push(Router::with_path("hello").get(hello)) // http://127.0.0.1:7878/hello?id=123
        .push(Router::with_path("resp").get(show_resp)) // http://127.0.0.1:7878/resp
        .push(Router::with_path("users/<id>").get(show).post(edit)) // http://127.0.0.1:7878/users/95
        .push(Router::new().path("custom_err").get(handle_custom)); // http://127.0.0.1:7878/custom_err

    let catchers: Vec<Box<dyn Catcher>> = vec![Box::new(Handle404)]; // catchers 错误页面
    let service = Service::new(router).with_catchers(catchers); // 设置 router 和 catcher
    Server::new(TcpListener::bind(addr)).serve(service).await;
}
