use std::fmt;
use std::error;
use super::server::*;

pub struct Command<'a, 'b> {
    name: &'a str,
    arity: u32,
    commandProc: fn(&'b Client, Vec<&str>) -> (),
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

pub fn process_command(client: &mut Client, buffer: String) -> () {
    let split_cmd: Vec<&str> = buffer.split_whitespace().collect();
    if split_cmd.len() < 1 {
        client.write_response(CRLF)
    }
}

// We disregard args here
pub fn ping_command(client: &mut Client, _args: Vec<&str>) -> () {
    client.write_response("PONG");
}

// We disregard args here
pub fn quit_command(_client: &mut Client, _args: Vec<&str>) -> () {
    //client.tear_down();
}

pub fn config_command(client: &mut Client, args: Vec<&str>) -> () {
    match args.get(0) {
        Some(&"get") => config_get_command(client, args[0]),
        Some(&"set") => config_set_command(client, args[1..].to_vec()),
        Some(arg) => client.write_response(format!("Unknown arg {}", arg).as_str()),
        None => client.write_response(format!("CONFIG requires arg GET|SET...").as_str()),
    }
}

fn config_get_command(client: &mut Client, key: &str) -> () {
    client.write_response(
        get_server_config(&client.server, key)
    );
}

fn config_set_command(client: &mut Client, args: Vec<&str>) -> () {
    client.write_response(format!("set {:?}", args).as_str());
}