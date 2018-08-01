use std::fmt;
use std::error;
use std::sync::{Arc, RwLock};
use super::server::*;

pub struct Command<'a, 'b> {
    name: &'a str,
    arity: u32,
    command_proc: fn(&'b Client, Vec<&str>) -> (),
}

pub struct Commands<'a, 'b>(Vec<Command<'a, 'b>>);

#[derive(Debug, Clone)]
pub struct CommandError;

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid command")
    }
}

impl error::Error for CommandError {
    fn description(&self) -> &str {
        return "Invalid command"
    }

    fn cause(&self) -> Option<&error::Error> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub fn process_command<'a>(server: &'a Arc<RwLock<ArcServer>>, input: Vec<&str>) -> ClientResponse {

    let cresp = match input[0] {
        "ping" => ping_command(),
        "config" => config_command(server, input),
        _ => ClientResponse {
                code: ARC_ERR,
                message: format!("unknown command `{}`", input[0]),
            },
    };
    cresp
}

// We disregard args here
pub fn ping_command() -> ClientResponse{
    ClientResponse {
        code: ARC_OK,
        message: "pong".to_string(),
    }
}

pub fn config_command<'a>(server: &'a Arc<RwLock<ArcServer>>, args: Vec<&str>) -> ClientResponse {
    match args.get(1) {
        Some(&"get") => config_get_command(server, args[2]),
        Some(arg) => ClientResponse {
            code: ARC_ERR,
            message: format!("unknown arg `{}`", arg),
        },
        None => ClientResponse {
            code: ARC_ERR,
            message: "CONFIG requires arg GET...".to_string(),
        }
    }
}

fn config_get_command<'a>(server: &'a Arc<RwLock<ArcServer>>, key: &str) -> ClientResponse {
    let handle = server.read().unwrap();
    let resp = match handle.config.get(key) {
        Some(val) => ClientResponse {
            code: ARC_OK,
            message: val,
        },
        None => ClientResponse {
            code: ARC_ERR,
            message: "unknown config key".to_string(),
        },
    };
    resp
}
