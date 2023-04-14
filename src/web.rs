#![allow(dead_code)]

use std::{fs, thread};
use std::net::TcpListener;
use std::net::TcpStream;
use std::io::prelude::*;
use std::time::Duration;
use rust_web::ThreadPool;

// 启动 web 服务
pub fn run_web_server(){
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    handle_tcp_listener_use_thread_pool(listener);
}

// 使用线程池是处理 tcp 连接
fn handle_tcp_listener_use_thread_pool(listener: TcpListener){

    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    println!("Shutting down.");
}

// 使用 stream流 是处理 tcp 连接
fn handle_tcp_listener(listener: TcpListener){
    for stream in listener.incoming()  {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream){
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        // 首页
        ("HTTP/1.1 200 OK", "hello.html")
    } else if buffer.starts_with(sleep) {
        // sleep 1 秒中后打开页面
        thread::sleep(Duration::from_secs(1));
        ("HTTP/1.1 200 OK", "hello.html")
    } else {
        // 其他页面
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();

}