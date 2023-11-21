use std::io::{BufReader, BufRead, Read};
use std::net::TcpStream;
use std::collections::HashMap;

#[derive(Debug)]
pub enum RequestType {
    GET, POST,
}

#[derive(Debug)]
pub struct Request {
    pub meth: RequestType,
    pub path: String,
    pub attr: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request
{
    pub fn new(reader: &mut BufReader<&mut TcpStream>) -> Option<Request>
    {
        let mut req = Request {
            meth: RequestType::GET,
            path: String::new(),
            attr: HashMap::<String, String>::new(),
            body: Vec::<u8>::new(),
        };

        let mut http_line = String::new();
        reader.read_line(&mut http_line).ok()?;
        let v: Vec<&str> = http_line.split(' ').collect();
        req.meth = match *v.get(0)? {
            "GET" => RequestType::GET,
            "POST" => RequestType::POST,
            _ => return None,
        };

        req.path = v.get(1)?.to_string();

        loop {
            let mut line = String::new();
            reader.read_line(&mut line).ok()?;
            line.pop()?;
            line.pop()?;
            if line == "" { break }
            let v: Vec<&str> = line.split(": ").collect();
            req.attr.insert(v.get(0)?.to_string(), v.get(1)?.to_string());
        }

        if let Some(len) = req.attr.get("Content-Length") {
            let len = len.parse::<usize>().ok()?;
            if len == 0 { return Some(req) }
            req.body = vec![0; len];
            reader.read_exact(&mut req.body).ok()?;
        }

        Some(req)
    }
}

#[derive(Debug)]
pub struct Response {
    http_code: i32,
    attr: HashMap<String, String>,
    body: Vec<u8>,
}

