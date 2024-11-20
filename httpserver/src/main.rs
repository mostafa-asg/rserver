mod server;
mod router;

use http::{httprequest::HttpRequest, httpresponse::HttpResponse};
use server::Server;

fn hello(req: &HttpRequest) -> HttpResponse {    
    let body = b"Hello World!".to_vec();
    HttpResponse::new("200", None, Some(body))
}

fn main() {
    let bind_address = "127.0.0.1:8000";
    let server = Server::new(&bind_address);
    server.register_handler(String::from("/hello"), hello);    
    println!("Server is listening {}", bind_address);
    server.run();
}
