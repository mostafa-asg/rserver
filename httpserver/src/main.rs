mod server;
mod router;

use http::{httprequest::HttpRequest, httpresponse::HttpResponse};
use server::Server;

fn hello(req: &HttpRequest) -> HttpResponse {    
    let body = b"Hello World!".to_vec();
    HttpResponse::new("200", None, Some(body))
}

fn greeting(req: &HttpRequest) -> HttpResponse { 
    let username = req.path_params.get("name").unwrap();
    let body =  format!("Hello {}!", username);
    HttpResponse::new("200", None, Some(body.into_bytes()))
}

fn user_order_details(req: &HttpRequest) -> HttpResponse { 
    let user_id = req.path_params.get("user_id").unwrap();
    let order_id = req.path_params.get("order_id").unwrap();

    let body =  format!("UserId: {}, OrderId: {}", user_id, order_id);
    HttpResponse::new("200", None, Some(body.into_bytes()))
}

fn main() {
    let bind_address = "127.0.0.1:8000";
    let server = Server::new(&bind_address);

    server.get("/hello", hello);    
    server.get("/hello/{name}", greeting);
    server.get("/users/{user_id}/orders/{order_id}", user_order_details);    

    println!("Server is listening {}", bind_address);
    server.run();
}
