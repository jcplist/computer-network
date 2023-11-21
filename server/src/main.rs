//mod db;
//use crate::db::*;

//use std::thread;

mod http;
use crate::http::*;

use std::{
    io::{BufReader},
    net::{TcpListener},
};

fn main()
{
    let listener = TcpListener::bind("127.0.0.1:5000").unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut reader = BufReader::new(&mut stream);
        let req = Request::new(&mut reader);
        println!("{:?}", req);
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
