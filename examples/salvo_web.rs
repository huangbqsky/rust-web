use salvo::{macros::Extractible};
use salvo::prelude::*;
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
    res.set_status_error(StatusError::internal_server_error().with_summary("error when serialize object to json"));
  
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().init();
    let addr = "127.0.0.1:7878";
    println!("listening on {}", addr);
    // 路由
    let router = Router::new()
        .push(Router::new().get(index)) // http://127.0.0.1:7878
        .push(Router::with_path("hello").get(hello)) // http://127.0.0.1:7878/hello?id=123
        .push(Router::with_path("resp").get(show_resp)) // http://127.0.0.1:7878/resp
        .push(Router::with_path("users/<id>").get(show).post(edit)); // http://127.0.0.1:7878/users/95
    Server::new(TcpListener::bind(addr))
        .serve(router)
        .await;
}
