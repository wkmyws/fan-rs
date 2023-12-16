use std::process::exit;
mod modules;
use modules::cli;
use modules::server;
use modules::fan;

use crate::modules::temp;


fn main() {
    if cli::is_cli() { // cli
        // temp
        if cli::get_temp() {
            println!("{}", temp::get_temp().unwrap_or(0.0).to_string());
            exit(0);
        }
        // speed
        let mut speed: u8 = cli::get_speed();
        if cli::is_auto() {
            speed = modules::temp::get_fan_level();
        }
        match fan::fan(speed) {
            Ok(speed) => println!("set speed to {}", speed),
            Err(err) => println!("err:{}", err),
        }
    } else { // server
        server::server::main();
    }
    exit(0);
}
