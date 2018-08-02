extern crate rand;
extern crate regex;

mod server;
mod commands;
mod config;
mod database;


fn main() {
    server::start_and_run()
}
