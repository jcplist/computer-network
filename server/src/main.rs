use std::thread;

pub mod config;
pub mod http;
pub mod db;
mod handler;
use crate::handler::handle_client;
use crate::config::{IDENTITY_FILE, SECRET};

use std::{
    sync::Arc,
    fs::File,
    io::{BufReader, Read},
    net::{TcpListener, TcpStream},
};

use native_tls::{Identity, TlsAcceptor, TlsStream};

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
