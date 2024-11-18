use std::collections::HashMap;
use std::io::{Write, Result};

#[derive(Debug, PartialEq, Clone)]
pub struct HttpResponse<'a> {
    pub version: &'a str,
    pub status_code: &'a str,
    pub status_text: &'a str,
    pub headers: Option<HashMap<&'a str, &'a str>>,
    pub body: Option<Vec<u8>>,
}

impl <'a> Default for HttpResponse<'a> {
    fn default() -> Self {
        HttpResponse {
            version: "HTTP/1.1",
            status_code: "200",
            status_text: "OK",
            headers: None,
            body: None
        }
    }
}

impl<'a> HttpResponse<'a> {
    pub fn new(status_code: &'a str, 
               headers: Option<HashMap<&'a str, &'a str>>,
               body: Option<Vec<u8>>) -> Self {
        let mut response = HttpResponse::default();
        response.status_code = status_code;
        response.status_text = match response.status_code {
            "200" => "OK",
            "400" => "Bad Request",
            "404" => "Not Found",
            "500" => "Internal Server Error",
            _ => ""
        };
        response.headers = match headers {
            Some(_) => headers,
            None => {
                let mut headers = HashMap::new();
                headers.insert("Content-Type", "text/html");
                Some(headers)
            }
        };
        response.body = body;

        response
    }

    pub fn send_response(&self, write_stream: &mut impl Write) -> Result<()> {
        write!(write_stream, 
               "{} {} {}\r\n", 
               self.version, self.status_code, self.status_text)?;
        
        if let Some(headers) = &self.headers {
            for (key, value) in headers {
                write!(write_stream, "{}: {}\r\n", key, value)?;
            }
        }
        
        if let Some(headers) = &self.headers {
            if let Some(body) = &self.body {
                if !headers.contains_key("Content-Length") {
                    write!(write_stream, "{}: {}\r\n", "Content-Length", body.len())?
                }    
            }
        }
        write!(write_stream, "\r\n")?;

        if let Some(body) = &self.body {            
            write_stream.write_all(body)?;
        }
        write_stream.flush()?;

        Ok(())
    }
}
