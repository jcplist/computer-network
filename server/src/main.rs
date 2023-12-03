//mod db;
//use crate::db::*;

use std::thread;

pub mod config;
mod http;
use crate::http::*;
use crate::config::{IDENTITY_FILE, SECRET};

use std::{
    sync::Arc,
    fs::File,
    io::{BufReader, Read},
    net::{TcpListener, TcpStream},
};

use native_tls::{Identity, TlsAcceptor, TlsStream};

fn handle_client(mut stream: TlsStream<TcpStream>) {
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

fn main()
{
    let mut file = File::open(IDENTITY_FILE).unwrap();
    let mut pkcs12 = vec![];
    file.read_to_end(&mut pkcs12).unwrap();
    let pkcs12 = Identity::from_pkcs12(&pkcs12, &SECRET).unwrap();

    let acceptor = TlsAcceptor::new(pkcs12).unwrap();
    let acceptor = Arc::new(acceptor);

    let listener = TcpListener::bind("0.0.0.0:5000").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let acceptor = acceptor.clone();
                thread::spawn(move || {
                    let stream = acceptor.accept(stream).unwrap();
                    handle_client(stream);
                });
            }
            Err(_e) => { println!("Get Client Failed."); }
        }
    }
}

/*
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
*/
