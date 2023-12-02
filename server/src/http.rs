use std::io::{BufReader, BufRead, Read, Write};
use std::net::TcpStream;
use std::collections::HashMap;

use base64_url::{encode, decode};
use serde::{Serialize, Deserialize};
use serde_encrypt::{serialize::impls::BincodeSerializer, EncryptedMessage, traits::SerdeEncryptSharedKey};

use crate::config::*;

#[derive(Serialize, Deserialize)]
struct HashMapWrap(HashMap<String,String>);

impl SerdeEncryptSharedKey for HashMapWrap {
    type S = BincodeSerializer<Self>;
}

// should remove pub if not testing
pub fn encode_cookie(src: &HashMap<String,String>) -> String
{
    encode(&HashMapWrap(src.clone()).encrypt(&KEY).unwrap().serialize())
}

// should remove pub if not testing
pub fn decode_cookie(src: String) -> Option<HashMap<String,String>>
{
    Some(HashMapWrap::decrypt_owned(&EncryptedMessage::deserialize(decode(&src).ok()?).ok()?, &KEY).ok()?.0)
}

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
    pub cookie: HashMap<String, String>
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
            cookie: HashMap::<String, String>::new(),
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

        if let Some(cookie_raw) = req.attr.get("Cookie") {
            let cookies: Vec<&str> = cookie_raw.split("; ").collect();
            for cookie_var in cookies {
                if cookie_var.len() > 6 && &cookie_var[..5] == "peach" {
                    let cookie_var = &cookie_var[6..].to_string();
                    if let Some(cookie) = decode_cookie(cookie_var.to_string()) {
                        if cookie.get("secret") == Some(&SECRET.to_string()) {
                            req.cookie = cookie;
                        }
                    }
                }
            }
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
    cookie: HashMap<String, String>,
    body: Vec<u8>,
}

impl Response {
    pub fn new() -> Response
    {
        Response {
            http_code: 0,
            attr: HashMap::<String, String>::new(),
            cookie: HashMap::<String, String>::new(),
            body: Vec::<u8>::new(),
        }
    }

    pub fn code(&mut self, x: i32)
    {
        self.http_code = x;
    }

    pub fn set(&mut self, key: &str, value: &str)
    {
        self.attr.insert(key.to_string(), value.to_string());
    }

    pub fn set_cookie(&mut self, key: &str, value: &str)
    {
        self.cookie.insert(key.to_string(), value.to_string());
    }

    pub fn data(&mut self, x: &Vec<u8>)
    {
        self.body = x.clone();
    }

    pub fn submit(&mut self, writer: &mut TcpStream) -> Option<()>
    {
        self.set_cookie("secret", SECRET);
        self.attr.insert("Set-Cookie".to_string(), format!("peach={}", encode_cookie(&self.cookie)));
        self.attr.insert("Content-Length".to_string(), self.body.len().to_string());
        self.set("Connection", "Closed");
        if self.attr.get("Content-Type") == None {
            self.set("Content-Type", "text/html; charset=utf-8");
        }
        let mut x = br#"HTTP/1.1 "#.to_vec();
        x.extend(self.http_code.to_string().as_bytes().to_vec());
        x.push(b' ');
        x.extend( match self.http_code {
            200 => "OK",
            206 => "Paartial Content",
            302 => "Found",
            400 => "Bad Request",
            401 => "Unauthorized",
            403 => "Forbidden",
            404 => "Not Found",
            _ => "qwq",
        }.as_bytes().to_vec());
        x.extend(b"\r\n".to_vec());
        for (key, value) in self.attr.iter(){
            x.extend(format!("{key}: {value}\r\n").as_bytes().to_vec());
        }
        x.extend(b"\r\n".to_vec());
        x.append(&mut self.body);
        writer.write_all(&x).ok()
    }
}
