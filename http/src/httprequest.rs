use core::str;
use std::{collections::HashMap, fmt::Display};

#[derive(Debug, PartialEq)]
pub enum Method {
    Get,
    Post,
    Uninitialized
}

#[derive(Debug, PartialEq)]
pub enum Version {
    V1_1,
    Uninitialized
}

#[derive(Debug, PartialEq)]
pub struct HttpRequest {
    pub version: Version,
    pub method: Method,
    pub resource: String,
    pub header: HashMap<String, String>,
    pub body: Vec<u8>
}

impl HttpRequest {
    pub fn parse(raw_request: Vec<u8>) -> Option<HttpRequest> {
        let end_of_header = raw_request.windows(4)
                                              .position(|window| window == b"\r\n\r\n")?;
        let (header_part, body_part) = raw_request.split_at(end_of_header + 4);
        let header_part = str::from_utf8(&header_part).ok()?;
        let mut lines = header_part.split("\r\n");
        
        // first line
        let first_header_line = lines.next()?;
        let mut first_line_parts = first_header_line.split_whitespace();
        let method = first_line_parts.next()?;
        let resource = first_line_parts.next()?;
        let version = first_line_parts.next()?;
    
        let mut headers = HashMap::new();
        for line in lines {
            if line.is_empty() {
                break;
            }
            let splittable = line.split_once(": ");
            match splittable {
                Some((key, value)) => headers.insert(key.to_owned(), value.to_owned()),              
                None => break
            };            
        }
    
        Some(HttpRequest {
            version: version.into(),
            method: method.into(),
            resource: resource.to_owned(),
            header: headers,
            body: body_part.to_vec(),
        })
    }        
}

impl From<&str> for Version {
    fn from(s: &str) -> Self {
        match s {
            "HTTP/1.1" => Version::V1_1,
            _ => Version::Uninitialized
        }
    }
}

impl From<&str> for Method {
    fn from(s: &str) -> Self {
        match s {
            "GET" => Method::Get,
            "POST" => Method::Post,
            _ => Method::Uninitialized
        }
    }
}

impl Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Self::Get => "GET",
            Self::Post => "POST",
            Self::Uninitialized => "Unknown",
        };
        write!(f, "{}", repr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_into_for_method() {
        let cases = vec![
            ("GET", Method::Get),
            ("POST", Method::Post),
            ("Unkown", Method::Uninitialized),
        ];

        for (input, expected) in cases {
            let m: Method = input.into();
            assert_eq!(m, expected, "Failed on input: {}", input);
        }
    }

    #[test]
    fn test_method_into_for_version() {
        let v: Version = "HTTP/1.1".into();
        assert_eq!(v, Version::V1_1, "Can't convert HTTP/1.1 to Version")
    }

    #[test]
    fn test_parse_http_request_empty_body() {
        let mut raw_request = Vec::new();
        raw_request.extend_from_slice(b"POST /submit-form HTTP/1.1\r\n");
        raw_request.extend_from_slice(b"Host: www.example.com\r\n");
        raw_request.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
        raw_request.extend_from_slice(b"Content-Length: 27\r\n\r\n");
        
        let result = HttpRequest::parse(raw_request).unwrap();
        let mut expected_headers = HashMap::new();
        expected_headers.insert(String::from("Host"), String::from("www.example.com"));
        expected_headers.insert(String::from("Content-Type"), String::from("application/x-www-form-urlencoded"));
        expected_headers.insert(String::from("Content-Length"), String::from("27"));

        assert_eq!(result.method, "POST".into());
        assert_eq!(result.resource, "/submit-form");
        assert_eq!(result.version, "HTTP/1.1".into());
        assert_eq!(result.header, expected_headers);
        assert_eq!(result.body, Vec::new());
    }

    #[test]
    fn test_parse_http_request_with_body() {
        let mut raw_request = Vec::new();
        raw_request.extend_from_slice(b"POST /submit-form HTTP/1.1\r\n");
        raw_request.extend_from_slice(b"Host: www.example.com\r\n");
        raw_request.extend_from_slice(b"Content-Type: application/x-www-form-urlencoded\r\n");
        raw_request.extend_from_slice(b"Content-Length: 27\r\n\r\n");
        raw_request.extend_from_slice(b"field1=value1&field2=value2");
        
        let result = HttpRequest::parse(raw_request).unwrap();
        let mut expected_headers = HashMap::new();
        expected_headers.insert(String::from("Host"), String::from("www.example.com"));
        expected_headers.insert(String::from("Content-Type"), String::from("application/x-www-form-urlencoded"));
        expected_headers.insert(String::from("Content-Length"), String::from("27"));
        let mut body = Vec::new();
        body.extend_from_slice(b"field1=value1&field2=value2");

        assert_eq!(result.method, "POST".into());
        assert_eq!(result.resource, "/submit-form");
        assert_eq!(result.version, "HTTP/1.1".into());
        assert_eq!(result.header, expected_headers);
        assert_eq!(result.body, body);
    }
}