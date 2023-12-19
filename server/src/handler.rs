use std::collections::{HashMap, HashSet};
use std::time::SystemTime;
use std::sync::Mutex;
use std::fs::File;
use std::io::{Read, BufReader, Seek, SeekFrom};
use std::net::TcpStream;
use std::cmp::min;

use native_tls::TlsStream;
use regex::Regex;

use crate::http::{Request, RequestType, Response};
use crate::db::*;

static COOKIE_ID: Mutex<i32> = Mutex::new(0);

use lazy_static::lazy_static;

lazy_static! {
    static ref VALID_COOKIE: Mutex<HashSet<String>> = Mutex::new(HashSet::new());
    static ref BOARD: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

fn get_handler(req: &mut Request, res: &mut Response) {
    if req.path == "/" {
        let mut f = File::open("./static/index.html").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        res.code(200);
        res.data(&buf);
    } else if req.path == "/auth" {
        if let Some(id) = req.cookie.get("id") {
            let ok = VALID_COOKIE.lock().unwrap();
            if ok.contains(id) {
                res.code(200);
                res.data(&req.cookie.get("name").unwrap().as_bytes().to_vec());
            } else {
                res.code(400);
            }
        } else {
            res.code(400);
        }
    } else if req.path == "/board" {
        if let Some(id) = req.cookie.get("id") {
            let ok = VALID_COOKIE.lock().unwrap();
            if ok.contains(id) {
                res.code(200);
                let board = BOARD.lock().unwrap();
                let board_data = board.join("\0");
                res.data(&board_data.as_bytes().to_vec());
            } else {
                res.code(400);
            }
        } else {
            res.code(400);
        }
    } else if req.path == "/login" {
        if let Some(id) = req.cookie.get("id") {
            let ok = VALID_COOKIE.lock().unwrap();
            if ok.contains(id) {
                res.code(302);
                res.set("Location", "/");
                return;
            }
        }
        let mut f = File::open("./static/login.html").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        res.code(200);
        res.data(&buf);
    } else if req.path == "/register" {
        if let Some(id) = req.cookie.get("id") {
            let ok = VALID_COOKIE.lock().unwrap();
            if ok.contains(id) {
                res.code(302);
                res.set("Location", "/");
                return;
            }
        }
        let mut f = File::open("./static/register.html").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        res.code(200);
        res.data(&buf);
    } else if req.path == "/bulletin" {
        if let Some(id) = req.cookie.get("id") {
            let ok = VALID_COOKIE.lock().unwrap();
            if ok.contains(id) {
                let mut f = File::open("./static/bulletin.html").unwrap();
                let mut buf = Vec::new();
                f.read_to_end(&mut buf).unwrap();
                res.code(200);
                res.data(&buf);
            } else {
                res.code(400);
            }
        } else {
            res.code(400);
        }
    } else if req.path == "/favicon.ico" {
        let mut f = File::open("./static/rngbased.jpg").unwrap();
        let mut buf = Vec::new();
        f.read_to_end(&mut buf).unwrap();
        res.code(200);
        res.set("Content-Type", "image/jpeg");
        res.data(&buf);
    } else if req.path == "/70000.mp4" {
        let mut f = File::open("./static/70000_10min.mp4").unwrap();

        let len = f.metadata().unwrap().len();
        let mut start = 0;
        let mut end = min(start + 100000, len);
        if let Some(range_raw) = req.attr.get("Range") {
            let r = Regex::new(r"bytes=(\d+)-(\d*)").unwrap();
            let caps = r.captures(range_raw).unwrap();
            start = caps.get(1).unwrap().as_str().parse().unwrap();
            if caps.get(2).unwrap().as_str() == "" {
                end = len;
            } else {
                end = caps.get(2).unwrap().as_str().parse().unwrap();
            }
            end = min(end, len);
            start = min(start, end);
            assert!(start < end);
            end = min(end, start + 1000000);
            res.code(206);
            f.seek(SeekFrom::Start(start)).unwrap();
            let mut buf = vec![0; (end - start) as usize];
            f.read_exact(&mut buf).unwrap(); 
            res.data(&buf);
            res.set("Content-Range", format!("bytes {}-{}/{}", start, end - 1, len).as_str());
        } else {
            end = len;
            res.code(200);
        }
        res.set("Accept-Ranges", "bytes");
        res.set("Content-Type", "video/mp4");
    }
}

fn post_handler(req: &mut Request, res: &mut Response) {
    if req.path == "/doLogin" {
        let s = std::str::from_utf8(&req.body).unwrap();
        let r = Regex::new(r"username=(\w+)&password=(\w+)").unwrap();
        let caps = r.captures(s).unwrap();
        let name = caps.get(1).unwrap().as_str();
        let pass = caps.get(2).unwrap().as_str();
        if let Some(user) = get_user(name) {
            if user.password.as_str() == pass {
                res.code(302);
                res.set("Location", "/");
                res.set_cookie("name", name);
                let mut cur_id = COOKIE_ID.lock().unwrap();
                res.set_cookie("id", cur_id.to_string().as_str());
                let mut ok = VALID_COOKIE.lock().unwrap();
                ok.insert(cur_id.to_string());
                *cur_id += 1;
                return;
            }
        }
        res.code(400);
    } else if req.path == "/doRegister" {
        let s = std::str::from_utf8(&req.body).unwrap();
        let r = Regex::new(r"username=(\w+)&password=(\w+)").unwrap();
        let caps = r.captures(s).unwrap();
        let name = caps.get(1).unwrap().as_str();
        let pass = caps.get(2).unwrap().as_str();
        if add_user(name, pass) {
            res.code(302);
            res.set("Location", "/login");
        } else {
            res.code(400);
        }
    } else if req.path == "/doBoard" {
        let s = std::str::from_utf8(&req.body).unwrap();
        if s.contains('\0') {
            res.code(400);
            return;
        }
        if let Some(id) = req.cookie.get("id") {
            let ok = VALID_COOKIE.lock().unwrap();
            if ok.contains(id) {
                res.code(200);
                let mut board = BOARD.lock().unwrap();
                board.push(s.to_string());
            } else {
                res.code(400);
            }
        } else {
            res.code(400);
        }
    }
}

pub fn handle_client(mut stream: TlsStream<TcpStream>) {
    let mut reader = BufReader::new(&mut stream);
    let mut req = Request::new(&mut reader).unwrap();
    let mut res = Response::new();
    println!("{:?}", req);
    match req.meth {
        RequestType::GET => get_handler(&mut req, &mut res),
        RequestType::POST => post_handler(&mut req, &mut res),
    };
    println!("{:?}", res);
    res.submit(&mut stream);
}
