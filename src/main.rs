#![allow(dead_code, unused)]
mod utils;
mod thread;
mod error;
mod web;

use std::thread::{sleep, spawn};
use std::io::{Result, Error, ErrorKind, BufReader, Write, BufRead, copy};
use std::time::Duration;
use std::sync::mpsc::channel;
use std::fs::File;
use std::net::{TcpListener, TcpStream};


/*
《Rust 程序设计语言》最后实现了一个多线程 web server， 说是实现了优雅停机与清理，其实只是线程池的 drop ，
只能“在处理两个请求之后通过退出循环来停止 server”。

那么我们来实现一个更好的 http server ，有这些功能：
1.文件请求 http://127.0.0.1:20083/abc.html 发送当前目录下 abc.html 的内容
2.简单 query string 处理 /?sleep /abc.html?sleep 暂停4秒再发送响应
3.正确的退出 处理 /?quit /abc.html?quit 会退出程序

技术细节：
1.标准库的 TcpListener 是没有什么正常的手段停止的。根据 Graceful exit TcpListener.incoming() 只有两种手段
  1).non-blocking 忙等，在 WouldBlock 事件判断是否退出
  2).不可移植的 let fd = listener.as_raw_fd(); libc::shutdown(fd, libc::SHUT_RD);
2.我们用独立线程的 accept_loop 处理 listener.accept 将接入连接转发给主线程的 dispatch_loop。
3.dispatch_loop 接受并处理来自 channel 的连接消息与退出消息，收到退出消息时退出循环。
4.accept_loop 以及 listener 是主线程退出时杀掉的，不算 listener 的正常停止，仅仅是程序正常退出。
5.暂时不用线程池
 */
fn main() -> Result<()> {
    let (dispatch_sender, dispatch_receiver) = channel::<DispatchMessage>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port))?;

    let dispatch_sender1 = dispatch_sender.clone();
    
    let accept_loop = spawn(move || {
        while let Ok((stream, addr)) = listener.accept() {
            println!("TcpListener accept: {} ", addr);
            dispatch_sender1.send(DispatchMessage::Connected(stream)).unwrap();
        }
    });
    println!("server started at http://{}:{}/ serving files in {:?}", local_host, port, std::env::current_dir().unwrap_or_default());

    while let Ok(dispatch_message) = dispatch_receiver.recv() {
        match dispatch_message {
            DispatchMessage::Connected(stream) => {
                let dispatch_sender = dispatch_sender.clone();
                spawn(move || {
                    if let Ok(RequestResult::Quit) = handle_connection(stream) {
                        dispatch_sender.send(DispatchMessage::Quit).unwrap();
                    }
                });
            }
            DispatchMessage::Quit => { break; }
        }
    }

    // accept_loop.join();
    Ok(())
}

#[derive(Debug)]
enum DispatchMessage {
    Connected(TcpStream),
    Quit,
}

enum RequestResult {
    Ok,
    Quit,
}

fn handle_connection(mut stream: TcpStream) -> Result<RequestResult> {
    let mut str = String::new();
    BufReader::new(&stream).read_line(&mut str)?;

    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        return Err(Error::from(ErrorKind::InvalidInput));
    }
    let method = strsubs[0];
    let path = strsubs[1];

    println!("method: {method} , path:{path}");

    let (path, query) = match path.find("?") {
        Some(pos) => (&path[..pos], &path[(pos+1)..]),
        None => (path, ""),
    };

    if query == "sleep" {
        sleep(Duration::new(4, 0));
    }

    if path == "/" {
        write!(stream, "HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome</body></html>")?;
    } else {
        let relative_path = match path.strip_prefix("/") {
            Some(p) => p,
            None => path,
        };
        match File::open(relative_path) {
            Ok(mut f) => {
                write!(stream, "HTTP/1.1 200 OK\r\n\r\n")?;
                copy(&mut f, &mut stream)?;
            }
            Err(err) => {
                eprintln!("{:?}", err);
                write!(stream, "HTTP/1.1 404 NOT FOUND\r\n\r\n<html><body>Not Found {}</body></html>", path)?;
            }
        }
    }
    stream.flush()?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}

// fn main() {
//     println!("Hello, world!");
//     web::run_web_server();
// }



