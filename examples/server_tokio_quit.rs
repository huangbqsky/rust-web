use tokio::task::spawn;
use tokio::time::sleep;
use tokio::io::{Result, Error, ErrorKind, AsyncWriteExt, BufReader, AsyncBufReadExt, copy};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel as channel;
use tokio::fs::File;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;

/**
 * 异步优雅停机优化
 * 既然能够真正的停止 listener，那就不需要双循环了
 * 
 * 技术细节:
1.这里 kill_switch 用了 unbounded_channel 而不是 tokio::sync::oneshot::channel 是因为用起来太麻烦，
大概要 Arc<Mutex<Option<oenshot::Sender>>>，可以改下试试。
2.FutureExt::race 相当于js的 Promise.race，而 tokio::select! 更加底层功能更强
 */
#[tokio::main]
async fn main() -> Result<()> {
    let (kill_switch, mut kill_switch_receiver) = channel::<()>();

    let local_host = "127.0.0.1";
    let port = 20083;
    let listener = TcpListener::bind((local_host, port)).await?;
    let accept_loop = spawn(async move {
        select! {
            _ = async {
                while let Ok((stream, addr)) = listener.accept().await {
                    println!("TcpListener accept: {} ", addr);
                    let kill_switch = kill_switch.clone();
                    spawn(async move {
                        if let Ok(RequestResult::Quit) = handle_connection(stream).await {
                            kill_switch.send(()).unwrap();
                        }
                    });
                }
            } => {}
            _ = kill_switch_receiver.recv() => {}
        }
    });
    println!("server started at http://{}:{}/ serving files in {:?}", local_host, port, std::env::current_dir().unwrap_or_default());

    accept_loop.await?;
    Ok(())
}


enum RequestResult {
    Ok,
    Quit,
}

async fn handle_connection(mut stream: TcpStream) -> Result<RequestResult> {
    let mut str = String::new();
    BufReader::new(&mut stream).read_line(&mut str).await?;
    println!("request: {}", str.trim());
    let strsubs: Vec<_> = str.split(" ").collect();
    if strsubs.len() < 3 {
        return Err(Error::from(ErrorKind::InvalidInput));
    }
    let method = strsubs[0];
    let path = strsubs[1];
    println!("method: {method} , path: {path}");

    let (path, query) = match path.find("?") {
        Some(pos) => (&path[..pos], &path[(pos+1)..]),
        None => (path, ""),
    };

    if query == "sleep" {
        sleep(Duration::new(4, 0)).await;
    }

    if path == "/" {
        stream.write("HTTP/1.1 200 OK\r\n\r\n<html><body>Welcome Tokio Server</body></html>".as_bytes()).await?;
    } else {
        let relative_path = match path.strip_prefix("/") {
            Some(p) => p,
            None => path,
        };
        match File::open(relative_path).await {
            Ok(mut f) => {
                stream.write("HTTP/1.1 200 OK\r\n\r\n".as_bytes()).await?;
                copy(&mut f, &mut stream).await?;
            }
            Err(err) => {
                eprintln!("{}", err);
                stream.write(format!("HTTP/1.1 404 NOT FOUND\r\n\r\n<html><body>Not Found {}</body></html>", path).as_bytes()).await?;
            }
        }
    }
    stream.flush().await?;

    if query == "quit" {
        return Ok(RequestResult::Quit);
    }
    return Ok(RequestResult::Ok);
}
