extern crate pid_from_port;
use pid_from_port::pid_from_port;
use std::env;

fn main() {
    let port = match env::args().nth(1) {
        Some(port_str) => match port_str.parse::<u16>() {
            Ok(port) => port,
            Err(e)=> {
                println!("Unparseable port argument: {}. Try again?", e);
                std::process::exit(0)
            }
        },
        None => {
            println!("No port argument given. Try again?");
            std::process::exit(0)
        }
    };
    match pid_from_port(port) {
        Ok(pid) => println!("{}",pid),
        Err(e) => {
            println!("{}",e);
            std::process::exit(0)
        }
    }
}