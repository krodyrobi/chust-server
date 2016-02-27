extern crate rustc_serialize;
extern crate crypto;
extern crate rand;
extern crate getopts;

mod data_base;
mod user;
mod connection;

use std::sync::{Arc, Mutex};
use std::io::Write;
use std::net::TcpListener;
use std::thread;
use std::path::Path;
use std::env;
use getopts::Options;

use connection::{Request, Response};
use user::User;
use data_base::DataBase;

fn main() {
    let path = Path::new("users.json");
    let mut data_base = if path.is_file() {
        DataBase::new(path).unwrap()
    } else {
        DataBase::empty(path)
    };

    let mut data_base = Arc::new(Mutex::new(data_base));

    let (ip, port) = match get_options() {
        Ok((ip, host)) => (ip, host),
        Err(message) => {
            println!("{}", message);
            return;
        }
    };

    let host: &str = &format!("{}:{}", ip, port);
    let listener = TcpListener::bind(host).unwrap();

    println!("listening");
    
    for stream in listener.incoming() {
        thread::spawn(|| {
            let mut stream = stream.unwrap();
            stream.write(b"Hello World\r\n").unwrap();
        });
    }
}

fn get_options() -> Result<(String, String), String>{
    let mut args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optopt("i", "host-ip", "set server host IP (default 127.0.0.1)", "IP");
    opts.optopt("p", "port", "set server port (default 25658))", "PORT");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return Err(f.to_string())
    };

    if matches.opt_present("h") {
        let message = format!("Usage: {} [options]\n", program);
        let message = opts.usage(&message);
        return Err(message);
    }

    let ip = matches.opt_str("i").unwrap_or("127.0.0.1".to_string());
    let port = matches.opt_str("p").unwrap_or("25658".to_string());

    Ok((ip, port))
}
