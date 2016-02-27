use std::fs::File;
use std::path::Path;
use std::error::Error;
use std::io::Read;
use std::io::Write;

use rustc_serialize::json::{self, Json};

use super::user::User;

pub struct DataBase<'a> {
    users: Vec<User>,
    path: &'a Path
}

impl<'a> DataBase<'a> {
    pub fn new(path: &Path) -> Result<DataBase, Box<Error>> {
        let mut file = try!(File::open(path));
        let mut string = String::new();

        try!(file.read_to_string(&mut string));

        let users: Vec<User> = try!(json::decode(&string));

        Ok(DataBase {
            users: users,
            path: path
        })
    }

    pub fn empty(path: &Path) -> DataBase {
        DataBase {
            users: vec![],
            path: path
        }
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

    use rustc_serialize::json::{self, Json};

    use super::super::user::User;
    use super::DataBase;

    #[test]
    fn empty() {
        let user1 = User::new("test1", "password1");
        let user2 = User::new("test2", "password2");

        let string = json::encode(&vec![user1.clone(), user2.clone()]).unwrap();

        let path = Path::new("/tmp/users_list.json");
        let mut data_base = DataBase::empty(path);

        data_base.users.push(user1);
        data_base.users.push(user2);

        data_base.write();

        let mut result = String::new();
        File::open(path).unwrap().read_to_string(&mut result);

        assert_eq!(result, string);
    }

    #[test]
    fn new() {
        let user1 = User::new("test1", "password1");
        let user2 = User::new("test2", "password2");
        let users = vec![user1, user2];

        let string = json::encode(&users).unwrap();
        let path = Path::new("/tmp/users_list.json");

        File::create(path).unwrap().write_all(string.as_bytes());

        let data_base = DataBase::new(path).unwrap();

        assert_eq!(data_base.users, users);
    }
}
