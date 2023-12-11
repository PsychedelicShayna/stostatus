use std::process::exit;

mod api;
mod error;
mod gzip;
mod http;
mod search;

use api::ServerStatus;
use error::Error;

fn main() {
    let debug = false;

    let status = api::check_server_status();

    let message_prefix = "STO Server";

    let status = match status {
        Ok(status) => status,
        Err(e) => {
            if debug {
                println!("Error: {:?}", e);
            }

            println!("Cannot Get Server Status");

            return;
        }
    };

    match status {
        ServerStatus::Online => println!("{} Online", message_prefix),
        ServerStatus::Offline => println!("{} Offline", message_prefix),
        ServerStatus::Unknown(s) => {
            if debug {
                println!("Unknown: {}", s);
            } else {
                println!("{} Unknown", message_prefix);
            }

            exit(1)
        }
    };
}
