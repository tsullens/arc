use std::fmt;
use std::error;
use std::sync::Arc;
use std::collections::HashMap;
use super::server::*;

pub struct Command {
    name: &'static str,
    arity: usize,
    pub command_proc: fn(&mut Arc<ArcServer>, &Vec<&str>) -> ClientResponse,
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
        m.insert("set", Command{name: "set", arity: 3, command_proc: set_command,});
        m.insert("get", Command{name: "get", arity: 2, command_proc: get_command,});
        m.insert("del", Command{name: "del", arity: 2, command_proc: del_command,});
        m.insert("sadd", Command{name: "sadd", arity: 3, command_proc: sadd_command,});
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
pub fn process_command(server: &mut Arc<ArcServer>, input: &str) -> Result<ClientResponse, CommandError> {
    let split_cmd: Vec<&str> = input.split_whitespace().collect();
    let key = split_cmd.get(0).unwrap_or_else(|| &"");
    
    match COMMAND_SET.get(key) {
        Some(command) => {
            if split_cmd.len() < command.arity {
                return Err(CommandError(format!("not enough arguments for command {}",command.name)));
            }
            let res = (command.command_proc)(server, &split_cmd[1..].to_vec());
            Ok(res)
        },
        None => Err(CommandError("unknown command".to_string())),
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
pub fn ping_command(_server: &mut Arc<ArcServer>, _args: &Vec<&str>) -> ClientResponse {
    ClientResponse::Ok("pong".to_string())
}

pub fn config_command(server: &mut Arc<ArcServer>, args: &Vec<&str>) -> ClientResponse {
    match args.get(0) {
        Some(&"get") => config_get_command(server, args[1]),
        Some(arg) => ClientResponse::Err(format!("unknown arg `{}`", arg)),
        None => ClientResponse::Err("CONFIG requires arg GET...".to_string()),
    }
}

fn config_get_command(server: &mut Arc<ArcServer>, key: &str) -> ClientResponse {
    let resp = match server.config.get(key) {
        Some(val) => ClientResponse::Ok(val),
        None => ClientResponse::Err("unknown config key".to_string()),
    };
    resp
}

/*
 * Database Commands
 */
pub fn set_command(server: &mut Arc<ArcServer>, args: &Vec<&str>) -> ClientResponse {
    let key = args[0];
    let mut db_handle = server.db.write().unwrap();
    db_handle.update_string(key, args[1]);
    return ClientResponse::Ok(String::new())
}

pub fn get_command(server: &mut Arc<ArcServer>, args: &Vec<&str>) -> ClientResponse {
    let key = args[0];
    let db_handle = server.db.read().unwrap();
    
    match db_handle.get(key) {
        Some(val) => ClientResponse::Ok(val.to_string()),
        None => ClientResponse::Err("key not found".to_string()),
    }
}

/*
 * get_command receives an Option from the database... 
 * the same could be done here...
 */
pub fn del_command(server: &mut Arc<ArcServer>, args: &Vec<&str>) -> ClientResponse {
    let key = args[0];
    let mut db_handle = server.db.write().unwrap();
    
    match db_handle.delete(key) {
        Ok(()) => ClientResponse::Ok(String::new()),
        Err(err) => ClientResponse::Err(err.to_string()),
    }


}

pub fn sadd_command(server: &mut Arc<ArcServer>, args: &Vec<&str>) -> ClientResponse {
    let key =  args[0];
    let set = &args[1..].to_vec();
    let mut db_handle = server.db.write().unwrap();

    db_handle.update_or_insert_set(key, set);
    return ClientResponse::Ok(String::new())
}
