use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::Read;
use std::io::Write;
use std::collections::HashMap;

use rustc_serialize::json::{self, Json};

use super::user::User;

pub struct DataBase<'a> {
    users: HashMap<String, User>,
    path: &'a Path
}

impl<'a> DataBase<'a> {
    pub fn new(path: &Path) -> Result<DataBase, Box<Error>> {
        let mut file = try!(File::open(path));
        let mut string = String::new();

        try!(file.read_to_string(&mut string));

        let users: HashMap<String, User> = try!(json::decode(&string));

        Ok(DataBase {
            users: users,
            path: path
        })
    }

    pub fn empty(path: &Path) -> DataBase {
        DataBase {
            users: HashMap::new(),
            path: path
        }
    }

    pub fn add(&mut self, user: User) -> bool {
        if self.users.contains_key(&user.username) {
            false
        } else {
            self.users.insert(user.username.clone(), user);
            true
        }
    }

    pub fn get(&self, username: &str) -> Option<&User> {
        self.users.get(username)
    }

    pub fn write(&self) -> Result<(), Box<Error>> {
        let mut file = try!(File::create(self.path));
        let string = try!(json::encode(&self.users));

        try!(file.write_all(string.as_bytes()));

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::path::Path;
    use std::io::Read;
    use std::io::Write;
    use std::collections::HashMap;

    use rustc_serialize::json::{self, Json};

    use super::super::user::User;
    use super::DataBase;

    #[test]
    fn empty() {
        let user1 = User::new("test1", "password1");

        let mut hash_map = HashMap::new();
        hash_map.insert("test1".to_string(), user1.clone());

        let string = json::encode(&hash_map).unwrap();

        let path = Path::new("/tmp/users_list.json");
        let mut data_base = DataBase::empty(path);

        data_base.add(user1);

        data_base.write();

        let mut result = String::new();
        File::open(path).unwrap().read_to_string(&mut result);

        assert_eq!(result, string);
    }

    #[test]
    fn new() {
        let user1 = User::new("test1", "password1");
        let user2 = User::new("test2", "password2");

        let mut hash_map = HashMap::new();
        hash_map.insert("test1".to_string(), user1.clone());
        hash_map.insert("test2".to_string(), user2.clone());

        let string = json::encode(&hash_map).unwrap();

        let path = Path::new("/tmp/users_list.json");
        File::create(path).unwrap().write_all(string.as_bytes());

        let data_base = DataBase::new(path).unwrap();
        assert_eq!(data_base.users.get("test1").unwrap(), hash_map.get("test1").unwrap());
        assert_eq!(data_base.users.get("test2").unwrap(), hash_map.get("test2").unwrap());
    }

    #[test]
    fn add() {
        let user1 = User::new("test1", "password1");
        let user2 = User::new("test1", "password1");

        let mut data_base = DataBase::empty(Path::new("/tmp/users_list.json"));

        assert!(data_base.add(user1));
        assert!(!data_base.add(user2));
    }
}
