#[macro_use] extern crate lazy_static;
extern crate rand;
extern crate regex;

mod server;
mod commands;

use std::net::{TcpListener};
use std::thread;
use std::sync::{Arc, RwLock};
use server::*;

fn main() {
    let server = Server::new(server::DEFAULT_CONFIG_FILE).unwrap();
    // Ahhhhh
    // doing this to unlock the server struct and then drop the unlock
    let bind_addr = format!("{}:{}", 
                    get_server_config(&server, "address"),
                    get_server_config(&server, "port")
        );
    let tcp_server = TcpListener::bind(bind_addr).unwrap();

    for connection in tcp_server.incoming() {
        match connection {
            Ok(stream) => {
                let foo = Arc::clone(&server);
                thread::spawn(move|| {
                    match Client::new(&stream, foo) {
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
