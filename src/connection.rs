
#[derive(Clone, Debug, Eq, PartialEq, RustcDecodable, RustcEncodable)]
pub enum Request {
    Auth(String, String),
    Reg(String, String),
    Send(String)
}

#[derive(Clone, Debug, Eq, PartialEq, RustcDecodable, RustcEncodable)]
pub enum Response {
    Ok,
    Err(String)
}
