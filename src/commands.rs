use std::fmt;
use std::error;
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

pub fn process_command(client: &mut Client, buffer: String) -> () {
    let upper = buffer.to_uppercase();
    let split_cmd: Vec<&str> = upper.split_whitespace().collect();
    if split_cmd.len() < 1 {
        client.write_response(ARK_CRLF);
    } else {
        match split_cmd[0] {
            "PING" => ping_command(client, split_cmd),
            "QUIT" => quit_command(client, split_cmd),
            "CONFIG" => config_command(client, split_cmd),
            _ => client.write_response(
                &format!("{}{}", ARK_ERR, "Unkown command")
            ),
        }
    }

}

// We disregard args here
pub fn ping_command(client: &mut Client, _args: Vec<&str>) -> () {
    client.write_response("PONG");
}

// We disregard args here
pub fn quit_command(client: &mut Client, _args: Vec<&str>) -> () {
    //client.tear_down();
}

pub fn config_command(client: &mut Client, args: Vec<&str>) -> () {
    let resp = match args.get(1) {
        Some(&"GET") => config_get_command(client, args[2]),
        Some(&"SET") => config_set_command(client, args[2..].to_vec()),
        Some(arg) => format!("Unknown arg {}", arg),
        None => format!("CONFIG requires arg GET|SET..."),
    };
    client.write_response(&resp);
}

fn config_get_command(client: &Client, key: &str) -> String {
    get_server_config(&client.server, key)
}

fn config_set_command(client: &mut Client, args: Vec<&str>) -> String {
    match set_server_config(&mut client.server, args[0], args[1..].to_vec()) {
        Ok(()) => return String::from(ARK_OK),
        Err(err) => return format!("{} {}", ARK_ERR, err),
    }
}