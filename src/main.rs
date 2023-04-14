mod utils;
mod thread;
mod error;
mod web;

fn main() {
    println!("Hello, world!");
    web::run_web_server();
}


