use std::fmt;
use std::error;
use std::sync::Arc;
use std::collections::HashMap;
use super::server::*;

pub struct Command {
    name: &'static str,
    arity: usize,
    pub command_proc: fn(&Arc<ArcServer>, &Vec<&str>) -> ClientResponse,
}

lazy_static! {
    /*
     * I have no idea if this makes sense or if it is correct.
     * I'm not actually sure if this is thread safe either...
     * is it OK to have many threads use the same function pointer? idk
     */
    static ref COMMAND_SET: HashMap<&'static str, Command> = {
        let mut m = HashMap::new();
        m.insert("config", Command{name: "config", arity: 3, command_proc: config_command,});
        m.insert("ping", Command{name: "ping", arity: 1, command_proc: ping_command,});
        m
    };
}

#[derive(Debug, Clone)]
pub struct CommandError(String);

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
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

/*
 * To-Do: 
 * CommandError makes almost know sense here when I can return a ClientRepsonse with the message
 */
pub fn process_command(server: &Arc<ArcServer>, input: &str) -> Result<ClientResponse, CommandError> {
    let split_cmd: Vec<&str> = input.split_whitespace().collect();
    let key = split_cmd.get(0).unwrap_or_else(|| &"");
    
    match COMMAND_SET.get(key) {
        Some(command) => {
            if split_cmd.len() < command.arity {
                return Err(CommandError(format!("not enough arguments for command {}",command.name)));
            }
            let res = (command.command_proc)(&server, &split_cmd[1..].to_vec());
            Ok(res)
        },
        None => Err(CommandError(format!("unknown command"))),
    }
}

/*
 * Dead code
pub fn process_command<'a>(server: &'a Arc<ArcServer>, input: Vec<&str>) -> ClientResponse {

    let cresp = match input[0] {
        "ping" => ping_command(server, &input),
        "config" => config_command(server, &input),
        _ => ClientResponse {
                code: ARC_ERR,
                message: format!("unknown command `{}`", input[0]),
            },
    };
    cresp
}
*/
// We disregard args here
pub fn ping_command<'a>(_server: &'a Arc<ArcServer>, _args: &Vec<&str>) -> ClientResponse{
    ClientResponse {
        code: ARC_OK,
        message: "pong".to_string(),
    }
}

pub fn config_command<'a>(server: &'a Arc<ArcServer>, args: &Vec<&str>) -> ClientResponse {
    match args.get(0) {
        Some(&"get") => config_get_command(server, args[1]),
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

fn config_get_command<'a>(server: &'a Arc<ArcServer>, key: &str) -> ClientResponse {;
    let resp = match server.config.get(key) {
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
