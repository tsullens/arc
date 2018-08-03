extern crate rand;
extern crate regex;
#[macro_use]
extern crate lazy_static;

mod server;
mod commands;
mod config;
mod database;


fn main() {
    server::start_and_run()
}
