### Sample usage
```rust
use http::{httprequest::HttpRequest, httpresponse::HttpResponse};
use server::Server;

fn user_order_details_handler(req: &HttpRequest) -> HttpResponse { 
    let user_id = req.path_params.get("user_id").unwrap();
    let order_id = req.path_params.get("order_id").unwrap();

    let body =  format!("UserId: {}, OrderId: {}", user_id, order_id);
    HttpResponse::new("200", None, Some(body.into_bytes()))
}

fn main() {
    let bind_address = "127.0.0.1:8000";
    let server = Server::new(&bind_address);

    server.get("/users/{user_id}/orders/{order_id}", user_order_details_handler);    

    println!("Server is listening {}", bind_address);
    server.run();
}
```
### How to run?
modify the content of [main](https://github.com/mostafa-asg/rserver/blob/main/httpserver/src/main.rs) and call `cargo run`.

#### why?
**Implemented from scratch just to learn Rust!**
