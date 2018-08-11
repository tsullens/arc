use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::io::{BufRead, Write, BufReader, BufWriter};
use std::thread;
use std::error;
use rand::prelude::*;
use std::sync::{Arc, RwLock};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::fmt;
use super::commands::*;
use super::config::*;
use super::database::*;

pub const ARC_CRLF: &'static str = "\r\n";
// pub const ARC_NL: &'static str = "\n";
pub const ARC_OK: &'static str = "+OK";
pub const ARC_ERR: &'static str = "-ERR";

#[derive(Debug)]
pub enum ClientResponse {
    Ok(String),
    Err(String),
}

impl fmt::Display for ClientResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ClientResponse::Ok(m) => {
                // We have empty messages e.g. for `set` commands
                // this prevents an extra blank line from being sent in the response
                if m.is_empty() {
                    write!(f, "{}", ARC_OK)
                } else {
                    write!(f, "{}{}{}", ARC_OK, ARC_CRLF, m)
                }
            },
            // Should always provide an error message
            ClientResponse::Err(m) => write!(f, "{}{}{}", ARC_ERR, ARC_CRLF, m),
        }
    }
}

pub struct Client<'a> {
    stream_reader: BufReader<&'a TcpStream>,
    stream_writer: BufWriter<&'a TcpStream>,
    id: u32,
    remote_addr: SocketAddr,
    local_addr: SocketAddr,
}

impl<'a> Client<'a> {
    
    pub fn new(stream: &'a TcpStream) -> Result<Self, Box<error::Error>> {
        let raddr = stream.peer_addr()?;
        let laddr = stream.local_addr()?;
        let mut rng = thread_rng();
        
        let c = Client {
            stream_reader: BufReader::new(&stream),
            stream_writer: BufWriter::new(&stream),
            id: rng.gen::<u32>(),
            remote_addr: raddr,
            local_addr: laddr,
        };
        Ok(c)
    }

    pub fn handle_connection(mut self, server: &mut Arc<ArcServer>) {
        println!("New connection with client id {} from {} -> {}", self.id, self.remote_addr, self.local_addr);
        loop {
            let mut input = String::new();
            let c_resp: ClientResponse = match self.stream_reader.read_line(&mut input) {
                Ok(_) => {
                    input = input.to_lowercase();
                    input = input.trim().to_string();
                    println!("Received command {}: cid|{}", input, self.id);

                    // This is handled as a special case
                    if input == "quit" {
                        break;
                    }

                    /*
                     * To-Do:
                     * Think about this more
                     */
                    match process_command(server, &input) {
                        Ok(resp) => resp,
                        Err(err) => ClientResponse::Err(err.to_string())
                    }
                },  
                Err(_) => ClientResponse::Err("invalid input".to_string())
            };
            self.write_response(c_resp);
        }
        self.tear_down();
    }

    fn tear_down(mut self) -> () {
        self.write_response(ClientResponse::Ok("disconnecting. bye!".to_string()));

        let stream = self.stream_reader.into_inner();
        match stream.shutdown(Shutdown::Both) {
            Err(err) => println!("Error shutting down client {}: {}", self.id, err),
            _ => (),
        }
    }

    pub fn write_response(&mut self, response: ClientResponse) -> () {
        println!("Sending (unformatted) response `{:?}`: cid|{}",
                response,
                self.id
        );
        let formatted_response = format!("{}{}", response, ARC_CRLF);

        // TODO: add exception handling
        self.stream_writer.write(formatted_response.as_bytes()).unwrap();
        self.stream_writer.flush().unwrap();
    }
}

#[derive(Debug, Clone)]
pub struct ConfigError;

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid configuration")
    }
}

impl error::Error for ConfigError {
    fn description(&self) -> &str {
        return "Invalid configuration"
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

#[derive(Debug)]
pub struct ArcServer {
    pub config: Config,
    pub db: RwLock<Database>,
    pub key_count: Arc<AtomicUsize>,
}

impl ArcServer {
    pub fn new(config: Config, database: Database) -> Arc<Self> {
        Arc::new(
            ArcServer {
                config: config,
                db: RwLock::new(database),
                key_count: Arc::new(AtomicUsize::new(0)),
            }
        )
    }

    pub fn inc_key_count(&self) -> () {
        let _old_count = self.key_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn dec_key_count(&self) -> () {
        let _old_count = self.key_count.fetch_sub(1, Ordering::Relaxed);
    }
}

pub fn start_and_run() {
    let config = Config::init(None);
    let db = Database::init();

    let tcp_listener = &[
            &config.get("bind_address").unwrap(),
            ":",
            &config.get("port").unwrap()
        ].join("");
    {
        let arc_server = ArcServer::new(config, db);

        let tcp_server = TcpListener::bind(tcp_listener).unwrap();

        println!("Successfully listening on {}", tcp_listener);

        for connection in tcp_server.incoming() {
            match connection {
                Ok(stream) => {
                    let mut arc_server_clone = Arc::clone(&arc_server);
                    thread::spawn(move|| {
                        match Client::new(&stream) {
                            Ok(conn) => conn.handle_connection(&mut arc_server_clone),
                            Err(err) => println!("Failed to set up connection: {}", err),
                        };
                    });
                },
                Err(err) => println!("Error unwrapping connection: {}", err),
            }
        }

    };
}