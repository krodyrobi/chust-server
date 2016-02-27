use crypto::bcrypt::bcrypt;
use rand::{thread_rng, Rng};

#[derive(Clone, Debug, Eq, PartialEq, RustcDecodable, RustcEncodable)]
pub struct User {
    username: String,
    salt: [u8; 16],
    hash: [u8; 24]
}

impl User {
    pub fn new(username: &str, password: &str) -> User {
        let mut hash = [0u8; 24];
        let mut salt = [0u8; 16];

        thread_rng().fill_bytes(&mut salt);

        bcrypt(5, &salt, password.as_bytes(), &mut hash);

        User {
            username: username.to_string(),
            salt: salt,
            hash: hash
        }
    }

    pub fn auth(&self, password: &str) -> bool {
        let mut output = [0u8; 24];
        bcrypt(5, &self.salt, password.as_bytes(), &mut output);

        output == self.hash
    }
}


#[cfg(test)]
mod test {
    use super::User;

    #[test]
    fn auth() {
        let user = User::new("rada", "password");

        assert!(user.auth("password"));
        assert!(!user.auth("password22"));
    }
}
