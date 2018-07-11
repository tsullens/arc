use std::net::{TcpStream, SocketAddr, Shutdown};
use std::io::{Read, BufRead, Write, BufReader, BufWriter};
use std::error;
use rand::prelude::*;
use std::fs::File;
use std::sync::{Arc, RwLock};
use std::fmt;
use super::commands;

pub const CRLF: &str = "\r\n";
pub const OK: &str = "+OK\r\n";
pub const ERR: &str = "-ERR\r\n";

pub const DEFAULT_CONFIG_FILE: &str = "./settings.conf";
const DEFAULT_ADDRESS_VAL: &str = "127.0.0.1";
const DEFAULT_PORT_VAL: &str = "7878";
const DEFAULT_SYNCWRITE_VAL: u8 = 1;

lazy_static! {
    static ref confMutKeys: Vec<&'static str> = vec![
        "syncwrite",
    ];

    static ref confKeys: Vec<&'static str> = {
        let mut v = vec![
            "address",
            "port",
            "debug",
        ];
        v.extend_from_slice(&confMutKeys);
        v
    };
}

pub struct Client<'a, 'b> {
    pub server: Arc<RwLock<Server<'b>>>,
    stream_reader: BufReader<&'a TcpStream>,
    stream_writer: BufWriter<&'a TcpStream>,
    id: u32,
    remote_addr: SocketAddr,
    local_addr: SocketAddr,
}

impl<'a, 'b> Client<'a, 'b> {
    
    pub fn new(stream: &'a TcpStream, server: Arc<RwLock<Server<'b>>>) -> Result<Client<'a, 'b>, Box<error::Error>> {
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
            
           commands::process_command(&mut self, input);
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
pub struct Server<'a> {
    address: &'a str,
    port: &'a str,
    isloaded: bool,
    debug: bool,
    syncwrite: u8,
}

impl<'a> Server<'a> {
    pub fn new(conff: &str) -> Result<Arc<RwLock<Server<'a>>>, ConfigError> {
        let mut server = Server {
            address: DEFAULT_ADDRESS_VAL,
            port: DEFAULT_PORT_VAL,
            isloaded: false,
            debug: true,
            syncwrite: DEFAULT_SYNCWRITE_VAL,
        };
        match server.processConfFile(conff) {
            Ok(()) => {
                return Ok(Arc::new(RwLock::new(server)));
            },
            Err(err) => Err(err),
        }
    }

    fn processConfFile(&mut self, conff: &str) -> Result<(), ConfigError> {
        let mut f = File::open(conff).expect("Configuration file not found or cannot be opened.");
        let mut contents = String::new();
        f.read_to_string(&mut contents);

        for (_line_num, line) in contents.lines().enumerate() {
            let line = line.trim().to_uppercase();

            if line.starts_with("#") {
                continue
            }
            let args: Vec<String> = line.split_whitespace()
                                        .map(|str| str.to_owned())
                                        .collect();
            match self.set_key(&args[0], &args[1..].to_vec()) {
               Ok(()) => continue,
               Err(err) => return Err(err),
           }
        }
        Ok(())
    }

    // directive is the entire `key = val`
    pub fn set_key(&mut self, str_key: &str, args: &'a Vec<String>) -> Result<(), ConfigError> {
        // we are trying to change a config val that is unchangeable 
        // after startup.
        if self.isloaded && !confMutKeys.contains(&str_key) {
            return Err(ConfigError);
        }
        match str_key.to_lowercase().as_str()  {
            "address" => self.address = &args[0],
            "port" => self.port = &args[0],
            "syncwrite" => self.syncwrite = args[0].parse::<u8>().unwrap(),
            _ => return Err(ConfigError),
        };
        Ok(())
    }

    pub fn get_key(&self, key: &str) -> &str {
        let res = match key {
            "address" => self.address,
            "port" => self.port,
            "*" => self.get_config_all(),
            _ => "UNKNOWN",
        };
        let ret = format!("{}", res).as_str();
        ret
    }

    fn get_config_all(&self) -> &'a str {
        return "<*>";
    }
}