use std::{collections::HashMap, io::Read, net::{TcpListener, TcpStream}, sync::{Arc, RwLock}, thread};

use http::{httprequest::HttpRequest, httpresponse::HttpResponse};

pub type Handler = fn(&HttpRequest) -> HttpResponse;

pub struct Server<'a> {
    socket_addr: &'a str,
    handlers: Arc<RwLock<HashMap<String, Handler>>>
}

impl<'a> Server<'a> {
    pub fn new(socket_addr: &'a str) -> Self {
        Server {
            socket_addr,
            handlers: Arc::new(RwLock::new(HashMap::new()))
        }
    }   

    fn handle_connection(stream: &mut TcpStream, handlers: Arc<RwLock<HashMap<String, Handler>>>) {
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
                let registered_paths = handlers.read().unwrap();
                match registered_paths.get(&request.resource) {
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
                    let handlers = self.handlers.clone();
                    thread::spawn(move || Server::handle_connection(&mut stream, handlers));
                },
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            };            
        }
    }

    pub fn register_handler(&self, path: String, handler: Handler) {
        let mut map = self.handlers.write().unwrap();
        map.insert(path, handler);
    }
}