use std::fs::File;
use std::io::{BufRead, BufReader};

const DEFAULT_CONFIG_FILE: &'static str = "./settings.conf";
const DEFAULT_BIND_ADDRESS: &'static str = "127.0.0.1";
const DEFAULT_BIND_PORT: &'static str = "7878";
const DEFAULT_CACHE_WRITE_THROUGH: u8 = 1;
const DEFAULT_DEBUG: bool = true;

#[derive(Debug)]
pub struct Config {
    pub bind_address: String,
    pub port: String,
    pub cache_write_through: u8,
    pub debug: bool,
}

impl Config {
    
    pub fn init(conf_file: Option<&str>) -> Config {
        let file_name = conf_file.unwrap_or_else(|| DEFAULT_CONFIG_FILE);
        let file = File::open(file_name).expect("Configuration file not found or cannot be opened.");
        let mut buf_reader = BufReader::new(file);
        let mut line = String::new();

        let mut config = Config {
            bind_address: DEFAULT_BIND_ADDRESS.to_owned(),
            port: DEFAULT_BIND_PORT.to_owned(),
            cache_write_through: DEFAULT_CACHE_WRITE_THROUGH,
            debug: DEFAULT_DEBUG,
        };

        let mut line_idx = 1;
        while let Ok(i) = buf_reader.read_line(&mut line) {
            if i > 0 {
                if line.starts_with("#") {
                    continue;
                }
                line = line.to_lowercase();
                let args: Vec<&str> = line.split_whitespace()
                                            .take(2)
                                            .collect();
                // We expect 2 args
                if args.len() != 2 {
                    println!("Bad configuration on line {}: {:?}", line_idx, args);
                    continue;
                }

                match args[0] {
                    "bind_address" => config.bind_address = args[1].to_owned(),
                    "port" => config.port = args[1].to_owned(),
                    "cache_write_through" => {
                        config.cache_write_through = args[1].parse::<u8>().unwrap();                    },
                    _ => continue
                }
                line_idx += 1;
            } else {
                break;
            }
        }
        return config
    }

    pub fn get(&self, key: &str) -> Option<String> {

        match &key.to_lowercase().as_str() {
            &"bind_address" => return Some(self.bind_address.clone()),
            &"port" => return Some(self.port.clone()),
            &"cache_write_through" => return Some(self.cache_write_through.to_string()),
            &"debug" => return Some(self.debug.to_string()),
            _ => None,
        }
    }

}