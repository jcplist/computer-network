//mod db;
//use crate::db::*;

//use std::thread;

pub mod config;
mod http;
use crate::http::*;

use std::collections::HashMap;

use std::{
    io::{BufReader},
    net::{TcpListener},
};

fn main()
{
    let mut a = HashMap::<String, String>::new();
    a.insert("a".to_string(), "b".to_string());
    a.insert("c".to_string(), "d".to_string());

    let b = encode_cookie(&a);
    println!("{:?}", b);
    println!("{:?}", decode_cookie(b));


    let listener = TcpListener::bind("127.0.0.1:5000").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(&mut stream);
        let req = Request::new(&mut reader);
        println!("{:?}", req);
        let mut res = Response::new();
        res.code(200);
        res.set_cookie("peach", "cute");
        res.data(&"peach is very cute.\n".as_bytes().to_vec());
        println!("{:?}", res);
        res.submit(&mut stream);
    }
    /*
    let a = thread::spawn(|| {
        add_user("peach", "peach");
    });
    let b = thread::spawn(|| {
        add_user("papaya", "papaya");
    });
    let c = thread::spawn(|| {
        add_user("apple", "apple");
    });
    a.join().unwrap();
    b.join().unwrap();
    c.join().unwrap();
    println!("{:?}", get_user("apple"));
    println!("{:?}", get_user("peach"));
    println!("{:?}", get_user("papaya"));
    */
}
