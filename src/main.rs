#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate regex;

mod server;
mod commands;

use std::net::{TcpListener};
use std::thread;
use std::sync::Arc;
use server::*;

fn main() {
    let server = Server::new(server::DEFAULT_CONFIG_FILE).unwrap();
    let mut bind_addr = String::new();
    {
        // seems like a hack
        let s = server.read().unwrap();
        bind_addr.push_str(s.get_key("address"));
        bind_addr.push_str(s.get_key("port"));
    }
    let tcp_server = TcpListener::bind(bind_addr).unwrap();

    for connection in tcp_server.incoming() {
        match connection {
            Ok(stream) => {
                let server = Arc::clone(&server);
                thread::spawn(move|| {
                    match Client::new(&stream, server) {
                        // conn is not a ref and we consume it with this call
                        Ok(conn) => {
                            conn.handle_connection();
                        },
                        Err(err) => {
                            println!("Failed to set up connection: {}", err);
                        },
                    };
                });
            },
            Err(err) => println!("Error unwrapping connection: {}", err),
        }
    }
}
