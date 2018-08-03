use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use std::io::{BufRead, Write, BufReader, BufWriter};
use std::thread;
use std::error;
use rand::prelude::*;
use std::sync::{Arc};
use std::fmt;
use super::commands::*;
use super::config::*;
use super::database::*;

pub const ARC_CRLF: &'static str = "\r\n";
pub const ARC_OK: &'static str = "+OK";
pub const ARC_ERR: &'static str = "-ERR";

pub struct ClientResponse {
    pub code: &'static str,
    pub message: String,
}

impl fmt::Display for ClientResponse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}{}{}", self.code, ARC_CRLF, &self.message, ARC_CRLF)
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

    pub fn handle_connection(mut self, server: Arc<ArcServer>) {
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
                    match process_command(&server, &input) {
                        Ok(resp) => resp,
                        Err(err) => ClientResponse {
                            code: ARC_ERR,
                            message: err.to_string(),
                        }
                    }
                },  
                Err(_) => ClientResponse {
                            code: ARC_ERR,
                            message: "invalid input".to_string(),
                        }
            };
            self.write_response(c_resp);
        }
        self.tear_down();
    }

    fn tear_down(mut self) -> () {
        self.write_response(ClientResponse {
            code: ARC_OK,
            message: "disconnecting. bye!".to_string(),
        });

        let stream = self.stream_reader.into_inner();
        match stream.shutdown(Shutdown::Both) {
            Err(err) => println!("Error shutting down client {}: {}", self.id, err),
            _ => (),
        }
    }

    pub fn write_response(&mut self, response: ClientResponse) -> () {
        println!("Sending (unformatted) response `{} {}`: cid|{}",
                &response.code,
                &response.message,
                self.id
        );
        let formatted_response = format!("{}", response);

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
    pub isloaded: bool,
    pub db: Database,
}

impl ArcServer {
    pub fn new(config: Config, database: Database) -> Arc<Self> {
        Arc::new(
            ArcServer {
                config: config,
                isloaded: false,
                db: database,
            }
        )
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
                    let arc_server_clone = Arc::clone(&arc_server);
                    thread::spawn(move|| {
                        match Client::new(&stream) {
                            Ok(conn) => conn.handle_connection(arc_server_clone),
                            Err(err) => println!("Failed to set up connection: {}", err),
                        };
                    });
                },
                Err(err) => println!("Error unwrapping connection: {}", err),
            }
        }

    };
}