extern crate rand;
extern crate regex;

mod server;
mod commands;
mod config;


fn main() {
    server::start_and_run()
}
