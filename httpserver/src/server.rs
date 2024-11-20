use std::{io::Read, net::{TcpListener, TcpStream}, sync::{Arc, RwLock}, thread};

use http::{httprequest::HttpRequest, httpresponse::HttpResponse};
use crate::router::{Router, RouteHandler};

pub struct Server<'a> {
    socket_addr: &'a str,    
    router: Arc<RwLock<Router>>,
}

impl<'a> Server<'a> {
    pub fn new(socket_addr: &'a str) -> Self {
        Server {
            socket_addr,            
            router: Arc::new(RwLock::new(Router::default()))
        }
    }   

    fn handle_connection(stream: &mut TcpStream, router: Arc<RwLock<Router>>) {
        let mut raw_request: Vec<u8> = Vec::new();
        let mut temp_buff = [0u8; 1024];
        loop {
            match stream.read(&mut temp_buff) {
                Ok(0) => break,
                Ok(n) => {
                    raw_request.extend_from_slice(&temp_buff);
                    if n < temp_buff.len() {
                        break;
                    }
                },
                Err(_) => break,
            }
        }

        let http_parse_result = HttpRequest::parse(raw_request);
        match http_parse_result {
            Some(request) => {                                
                match router.read().unwrap().find_handler(request.method, &request.resource) {
                    Some(handler) => {
                        let response = handler(&request);                        
                        response.send_response(stream).unwrap();
                    },
                    None => {
                        let not_found = HttpResponse::new("404", None, None);
                        not_found.send_response(stream).unwrap();        
                    }
                }
            },
            None => {
                let bad_request = HttpResponse::new("400", None, None);
                bad_request.send_response(stream).unwrap();
            }
        }        
    }

    pub fn run(&self) {
        let listener = TcpListener::bind(self.socket_addr)
                                        .expect(format!("Couldn't bind to address {}", self.socket_addr).as_str());
        for new_connection in listener.incoming() {
            match new_connection {
                Ok(mut stream) => {
                    let router = self.router.clone();
                    thread::spawn(move || Server::handle_connection(&mut stream, router));
                },
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            };            
        }
    }

    pub fn get(&self, path: &str, handler: RouteHandler) {
        let mut router = self.router.write().unwrap();
        router.get(path, handler);
    }

    pub fn post(&self, path: &str, handler: RouteHandler) {
        let mut router = self.router.write().unwrap();
        router.post(path, handler);
    }

}