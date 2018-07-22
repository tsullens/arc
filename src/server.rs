use std::net::{TcpStream, SocketAddr, Shutdown};
use std::io::{Read, BufRead, Write, BufReader, BufWriter};
use std::error;
use rand::prelude::*;
use std::fs::File;
use std::sync::{Arc, RwLock};
use std::fmt;
use super::commands::*;

pub const ARK_CRLF: &str = "\r\n";
pub const ARK_OK: &str = "+OK\r\n";
pub const ARK_ERR: &str = "-ERR\r\n";

pub const DEFAULT_CONFIG_FILE: &str = "./settings.conf";
const DEFAULT_ADDRESS_VAL: &str = "127.0.0.1";
const DEFAULT_PORT_VAL: &str = "7878";
const DEFAULT_SYNCWRITE_VAL: u8 = 1;

lazy_static! {
    static ref CONF_MUT_KEYS: Vec<&'static str> = vec![
        "syncwrite",
    ];

    static ref CONF_KEYS: Vec<&'static str> = {
        let mut v = vec![
            "address",
            "port",
            "debug",
        ];
        v.extend_from_slice(&CONF_MUT_KEYS);
        v
    };
}

pub struct Client<'a> {
    pub server: Arc<RwLock<ArkServer>>,
    stream_reader: BufReader<&'a TcpStream>,
    stream_writer: BufWriter<&'a TcpStream>,
    id: u32,
    remote_addr: SocketAddr,
    local_addr: SocketAddr,
}

impl<'a> Client<'a> {
    
    pub fn new(stream: &'a TcpStream, server: Arc<RwLock<ArkServer>>) -> Result<Client<'a>, Box<error::Error>> {
        let raddr = stream.peer_addr()?;
        let laddr = stream.local_addr()?;
        let mut rng = thread_rng();
        
        let c = Client {
            server: server,
            stream_reader: BufReader::new(&stream),
            stream_writer: BufWriter::new(&stream),
            id: rng.gen::<u32>(),
            remote_addr: raddr,
            local_addr: laddr,
        };
        Ok(c)
    }

    pub fn handle_connection(mut self) {
        println!("New connection with client id {} from {} -> {}", self.id, self.remote_addr, self.local_addr);
        loop {
            let mut input = String::new();
            match self.stream_reader.read_line(&mut input) {
                Ok(_) => {
                    println!("Received command {}: cid|{}", input.trim(), self.id);
                },
                Err(_) => {
                    self.write_response("Invalid input");
                    continue
                }
            };
            
           process_command(&mut self, input);
        }
    }

    pub fn tear_down(mut self) -> () {
        self.write_response("bye!");
        let stream = self.stream_reader.into_inner();
        match stream.shutdown(Shutdown::Both) {
            Err(err) => println!("Error shutting down client {}: {}", self.id, err),
            _ => (),
        }
    }

    pub fn write_response(&mut self, response: &str) -> () {
        println!("Sending response `{}`: cid|{}", response, self.id);
        let formatted_response = format!("{}\r\n", response);

        // TODO: add exception handling
        self.stream_writer
            .write(
                formatted_response.as_bytes()
            ).unwrap();
        self.stream_writer
            .flush()
            .unwrap();
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
pub struct ArkServer {
    address: String,
    port: String,
    isloaded: bool,
    debug: bool,
    cache_write_through: u8,
}

impl ArkServer {
    pub fn new(conf_f: &str) -> Result<Arc<RwLock<ArkServer>>, ConfigError> {
        let mut server = Arc::new(
                RwLock::new(
                    ArkServer {
                        address: String::from(DEFAULT_ADDRESS_VAL),
                        port: String::from(DEFAULT_PORT_VAL),
                        isloaded: false,
                        debug: true,
                        cache_write_through: DEFAULT_SYNCWRITE_VAL,
                    }));  
        
        match processConfFile(&mut server, conf_f) {
            Ok(()) => {
                return Ok(server);
            },
            Err(err) => Err(err),
        }
    }
}

fn processConfFile(mut server: &mut Arc<RwLock<ArkServer>>, conff: &str) -> Result<(), ConfigError> {
    let mut f = File::open(conff).expect("Configuration file not found or cannot be opened.");
    let mut contents = String::new();
    f.read_to_string(&mut contents);

    for (_line_num, line) in contents.lines().enumerate() {
        let line = line.trim().to_uppercase();

        if line.starts_with("#") {
            continue
        }
        let args: Vec<&str> = line.split_whitespace().collect();
        match set_server_config(&mut server, &args[0], args[1..].to_vec()) {
            Ok(()) => continue,
            Err(err) => return Err(err),
        }
    }
    Ok(())
}

// directive is the entire `key val [val..]`
pub fn set_server_config(server: &mut Arc<RwLock<ArkServer>>, key: &str, args: Vec<&str>) -> Result<(), ConfigError> {
    let mut s = server.write().unwrap();
    match key.to_uppercase().as_str() {
        "ADDRESS" => s.address = String::from(args[0]),
        "PORT" => s.port = String::from(args[0]),
        "CACHE_WRITE_THROUGH" => s.cache_write_through = args[0].parse::<u8>().unwrap(),
        _ => return Err(ConfigError),
    };
    Ok(())
}

pub fn get_server_config(server: &Arc<RwLock<ArkServer>>, key: &str) -> String {
    let s = server.read().unwrap();

    return match key.to_uppercase().as_str() {
        "ADDRESS" => String::clone(&s.address),
        "PORT" => String::clone(&s.port),
        "CACHE_WRITE_THROUGH" => s.cache_write_through.to_string(),
        _ => String::from("UNKNOWN"),
    };
}