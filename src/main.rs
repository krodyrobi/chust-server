extern crate rustc_serialize;
extern crate crypto;
extern crate rand;
extern crate getopts;

mod data_base;
mod user;
mod connection;

use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::thread;
use std::path::Path;
use std::env;
use getopts::Options;
use std::collections::HashMap;

use rustc_serialize::json;

use connection::{ClientRequest, Response};
use user::User;
use data_base::DataBase;

fn main() {
    let room = Arc::new(Mutex::new(HashMap::new()));

    let path = Path::new("users.json");
    let data_base = if path.is_file() {
        DataBase::new(path).unwrap()
    } else {
        DataBase::empty(path)
    };

    let data_base = Arc::new(Mutex::new(data_base));

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

    for tcp_stream in listener.incoming() {
        let room = room.clone();
        let data_base = data_base.clone();

        thread::spawn(move || {
            let mut current_user: Option<String> = None;
            let stream = tcp_stream.unwrap();
            let reader = Arc::new(Mutex::new(BufReader::new(stream.try_clone().unwrap())));

            loop {
                let mut string = String::new();
                match reader.lock().unwrap().read_line(&mut string) {
                    Ok(_) => (),
                    Err(_) => {
                        match current_user.clone() {
                            Some(user) => {
                                let mut userlist = room.lock().unwrap();
                                userlist.remove(&user);
                                println!("user {} disconnected", &user);
                            }
                            None => (),
                        }
                        stream.shutdown(std::net::Shutdown::Both).unwrap();
                    }
                }

                let response = match json::decode(&string) {
                    Ok(request) => {
                        match request {
                            ClientRequest::Auth(username, password) => {
                                let data_base = data_base.lock().unwrap();

                                match data_base.get(&username) {
                                    Some(user) => {
                                        if user.auth(&password) {
                                            // Authenticate user
                                            current_user = Some(username.clone());

                                            // Add to userlist
                                            let mut userlist = room.lock().unwrap();
                                            userlist.insert(username.clone(),
                                                            BufReader::new(stream.try_clone()
                                                                                 .unwrap()));

                                            Response::Ok
                                        } else {
                                            Response::Err(2,
                                                          "Incorrect username or password."
                                                              .to_string())
                                        }
                                    }
                                    None => {
                                        Response::Err(2,
                                                      "Incorrect username or password.".to_string())
                                    }
                                }
                            }
                            ClientRequest::Reg(username, password) => {
                                let mut data_base = data_base.lock().unwrap();

                                let result = data_base.add(User::new(&username, &password));
                                data_base.write().unwrap();

                                if result {
                                    Response::Ok
                                } else {
                                    Response::Err(4, "User already registered".to_string())
                                }
                            }
                            ClientRequest::Send(message) => {
                                match current_user {
                                    Some(ref user) => {
                                        println!("got message {} from user {}", message, user);
                                        for (_, ref mut readr) in room.lock().unwrap().iter_mut() {
                                            let line = format!("{}\n",
                                                               user.to_string() + ": " + &message);
                                            readr.get_mut().write(line.as_bytes()).unwrap();
                                        }

                                        Response::Ok
                                    }
                                    None => Response::Err(3, "Authenticate first.".to_string()),
                                }
                            }
                        }
                    }
                    Err(_) => Response::Err(1, "Can't parse request.".to_string()),
                };

                let line = format!("{}\n", json::encode(&response).unwrap());

                reader.lock().unwrap().get_mut().write(line.as_bytes()).unwrap();
            }
        });
    }
}

fn get_options() -> Result<(String, String), String> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();

    opts.optopt("i",
                "host-ip",
                "set server host IP (default 127.0.0.1)",
                "IP");
    opts.optopt("p", "port", "set server port (default 25658))", "PORT");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => return Err(f.to_string()),
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
