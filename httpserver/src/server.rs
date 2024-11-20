use std::{collections::HashMap, io::Read, net::{TcpListener, TcpStream}, sync::{Arc, RwLock}, thread};

use http::{httprequest::HttpRequest, httpresponse::HttpResponse};
use crate::router::{Router, RouteHandler, normalize_path};

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

        fn extract_path_params(path: &str, params_pos: &HashMap<usize, String>) -> HashMap<String, String> {
            let normalized_path = normalize_path(path);
            let parts: Vec<&str> = normalized_path.split('/').collect();
            let mut result = HashMap::new();

            let s:usize = 12;
            for (position, param_name) in params_pos.iter() {
                let param_value = parts[*position];
                result.insert(param_name.to_owned(), param_value.to_owned());
            }

            return result;
        }

        let mut http_parse_result = HttpRequest::parse(raw_request);
        match http_parse_result {
            Some(ref mut request) => {                                
                match router.read().unwrap().find_handler(request.method, &request.resource) {
                    Some(route_info) => {
                        let handler = route_info.handler;                        
                        // extract path parameters
                        let path_params = extract_path_params(&request.resource, &route_info.params_pos);
                        request.with_path_params(&path_params);  

                        // execute the handler
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