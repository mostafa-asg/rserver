use std::collections::HashMap;
use indexmap::IndexMap;
use regex::Regex;

use http::httprequest::{HttpRequest, Method};
use http::httpresponse::HttpResponse;

pub type RouteHandler = fn(&HttpRequest) -> HttpResponse;

const CATCH_ALL: &str = "[^/]+"; // catch everything expect slash

/// Normalizes a path by removing leading and trailing slashes.
pub fn normalize_path(path: &str) -> &str {        
    if path.len() > 2 {
        let mut start: usize = 0;
        let mut end: usize = path.len();
        if path.starts_with('/') {
            start += 1;
        }
        if path.ends_with('/') {
            end -= 1;
        }
        &path[start..end]    
    } else {
        path
    }        
}

fn find_params(path: &str) -> HashMap<usize, String> {
    let normalized_path = normalize_path(path);
    let parts = normalized_path.split("/");
    let mut result:HashMap<usize, String> = HashMap::new();
    let mut position: usize = 0;
    for part in parts {
        if part.starts_with('{') && part.ends_with('}') && part.len() > 2 {
            let param_name = &part[1..part.len()-1];
            result.insert(position, param_name.to_string());
        }
        position+=1;
    }
    result
}

/// Build the regex experssion that matches the path
fn regex_that_match(path: &str) -> String {
    let params = find_params(path);
    let normalized_path = normalize_path(path);
    let parts:Vec<&str> = normalized_path.split('/').collect();
    let mut position: usize = 0;
    let mut regex_expersion_parts: Vec<String> = Vec::new();
    let part_size = parts.len();

    for part in parts {
        let item = match params.get(&position) {
            Some(_) => CATCH_ALL, 
            None => part, 
        };
        let mut regex_item = format!("{}", item);        
        if position == 0 {
            if path.starts_with('/') {
                regex_item = format!("^/{}", regex_item);
            } else {
                regex_item = format!("^{}", regex_item);
            }
        }
        if position == part_size-1 {
            if path.ends_with('/') {
                regex_item = format!("{}/$", regex_item);
            } else {
                regex_item = format!("{}$", regex_item);
            }
        }
        regex_expersion_parts.push(regex_item);
        position+=1;
    }

    regex_expersion_parts.join("/")
}

pub struct RouteInfo {
    pub handler: RouteHandler,
    pub params_pos: HashMap<usize, String>, // key: position, value: parameter name
}

pub struct Router {
    // Paths for Get method
    get_entries: IndexMap<String, RouteInfo>,

    // Paths for Post method
    post_entries: IndexMap<String, RouteInfo>,
}

impl Default for Router {
    fn default() -> Self {
        Router {
            get_entries: IndexMap::new(),
            post_entries: IndexMap::new(),
        }
    }
}

impl Router {
    pub fn get(&mut self, path: &str, handler: RouteHandler) {
        let regex_for_path = regex_that_match(path);
        let path_parameters = find_params(path);
        self.get_entries.insert(regex_for_path, RouteInfo{
            handler: handler,
            params_pos: path_parameters,
        });        
    }

    pub fn post(&mut self, path: &str, handler: RouteHandler) {
        let regex_for_path = regex_that_match(path);
        let path_parameters = find_params(path);
        self.post_entries.insert(regex_for_path, RouteInfo{
            handler: handler,
            params_pos: path_parameters,
        });        
    }

    pub fn find_handler(&self, method: Method, path: &str) -> Option<&RouteInfo> {
        let hashmap = match method {
            Method::Get => &self.get_entries,
            Method::Post => &self.post_entries,
            Method::Uninitialized => {
                return None;
            }
        };

        for (regex_exper, route_info) in hashmap {            
            let regex = Regex::new(regex_exper).unwrap();
            if regex.is_match(path) {                
                return Some(route_info);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {        
        assert_eq!(normalize_path("/home/"), "home");
        assert_eq!(normalize_path("/user"), "user");
        assert_eq!(normalize_path("root/"), "root");
        assert_eq!(normalize_path("a"), "a");
        assert_eq!(normalize_path("//"), "//");
    }

    #[test]
    fn test_find_params() {        
        let result = find_params("/users");
        assert_eq!(result.len(), 0);

        let result = find_params("/users/{user_id}");
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&1).expect("At position 1 there is a parameter"), "user_id");

        let result = find_params("/users/{user_id}/orders/{order_id}");
        assert_eq!(result.len(), 2);
        assert_eq!(result.get(&1).expect("At position 1 there is a parameter"), "user_id");
        assert_eq!(result.get(&3).expect("At position 3 there is a parameter"), "order_id");

        // check when parameter is at beginning
        let result = find_params("{weird}/users");
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&0).expect("At position 0 there is a parameter"), "weird");

        // catch all path
        let result = find_params("{*}");
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&0).expect("At position 0 there is a parameter"), "*");
    }

    #[test]
    fn test_regex_that_match_function() {        
        assert_eq!(regex_that_match("home"), "^home$");
        assert_eq!(regex_that_match("/home"), "^/home$");
        assert_eq!(regex_that_match("/home/"), "^/home/$");
        assert_eq!(regex_that_match("/users/{user_id}"), format!("^/users/{}$", CATCH_ALL));
        assert_eq!(regex_that_match("/users/{user_id}/orders/{order_id}"), format!("^/users/{}/orders/{}$", CATCH_ALL, CATCH_ALL));
    }

    #[test]
    fn test_path_matching() {
        let home_handler: RouteHandler = |_: &HttpRequest| -> HttpResponse {
            HttpResponse::default()
        };
        let all_users: RouteHandler = |_: &HttpRequest| -> HttpResponse {
            HttpResponse::default()
        };
        let user_activity: RouteHandler = |_: &HttpRequest| -> HttpResponse {
            HttpResponse::default()
        };
        let user_detail: RouteHandler = |_: &HttpRequest| -> HttpResponse {
            HttpResponse::default()
        };
        let user_all_orders: RouteHandler = |_: &HttpRequest| -> HttpResponse {
            HttpResponse::default()
        };
        let user_order_details: RouteHandler = |_: &HttpRequest| -> HttpResponse {
            HttpResponse::default()
        };

        let mut router = Router::default();
        router.get("/home", home_handler);
        router.get("/users", all_users);
        router.get("/activities/{user_id}", user_activity);
        router.get("/users/{user_id}", user_detail);
        router.get("/users/{user_id}/orders", user_all_orders);
        router.get("/users/{user_id}/orders/{order_id}", user_order_details);

        let handler = router.find_handler(Method::Get, "/users/123/orders");
        assert_eq!(handler.is_some(), true);
        assert_eq!(handler.unwrap().handler, user_all_orders);

        let handler = router.find_handler(Method::Get, "/users/123/orders/A123");
        assert_eq!(handler.is_some(), true);
        assert_eq!(handler.unwrap().handler, user_order_details);

        let handler = router.find_handler(Method::Get, "/users/user1");
        assert_eq!(handler.is_some(), true);
        assert_eq!(handler.unwrap().handler, user_detail);

        let handler = router.find_handler(Method::Get, "/users");
        assert_eq!(handler.is_some(), true);
        assert_eq!(handler.unwrap().handler, all_users);

        let handler = router.find_handler(Method::Get, "/user");
        assert_eq!(handler.is_some(), false);
    }
}