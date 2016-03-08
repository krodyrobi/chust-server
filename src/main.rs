extern crate rustc_serialize;
extern crate crypto;
extern crate rand;
extern crate getopts;

mod data_base;
mod user;
mod connection;

use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::thread;
use std::path::Path;
use std::env;
use std::str::from_utf8;
use getopts::Options;

use rustc_serialize::json::{self, Json};

use connection::{ClientRequest, ServerRequest, Response};
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
        let mut data_base = data_base.clone();

        thread::spawn(move || {
            let mut stream = stream.unwrap();
            let mut reader = BufReader::new(&stream);

            loop {
                let mut string = String::new();
                reader.read_line(&mut string).unwrap();

                let response = match json::decode(&string) {
                    Ok(request) => {
                        match request {
                            ClientRequest::Auth(username, password, port) => {
                                let data_base = data_base.lock().unwrap();

                                match data_base.get(&username) {
                                    Some(user) => {
                                        if user.auth(&password) {
                                            //TODO port + store
                                            Response::Ok
                                        } else {
                                            Response::Err(2, "Incorrect username or password.".to_string())
                                        }
                                    }
                                    None => Response::Err(2, "Incorrect username or password.".to_string())
                                }
                            }
                            ClientRequest::Reg(username, password) => {
                                Response::Ok
                            }
                            ClientRequest::Send(message) => {
                                Response::Ok
                            }
                        }
                    }
                    Err(_) => {
                        Response::Err(1, "Can't parse request.".to_string())
                    }
                };

                let line = format!("{}\n", json::encode(&response).unwrap());

                reader.get_mut().write(line.as_bytes()).unwrap();
            }
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
