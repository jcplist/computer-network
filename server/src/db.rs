use pickledb::{PickleDb, PickleDbDumpPolicy};
use serde::{Serialize, Deserialize};

pub trait Empty
{
    fn empty() -> Self;
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub username: String,
    pub password: String,
}

impl Empty for User
{
    fn empty() -> User
    {
        User { username: String::new(), password: String::new() }
    }
}

impl User
{
    pub fn new(username: &str, password: &str) -> User
    {
        User { username: username.to_string(), password: password.to_string() }
    }
}

pub struct Database<T> {
    handle: PickleDb,
    marker: T,
}

impl<'a, T: Serialize + Empty + Clone + for<'de> Deserialize<'de>> Database<T>
{
    pub fn new(filename: &str) -> Database<T>
    {
        Database
        {
            handle: PickleDb::load_json(filename, PickleDbDumpPolicy::AutoDump)
                .unwrap_or_else(|_| PickleDb::new_json(filename, PickleDbDumpPolicy::AutoDump)),
            marker: T::empty()
        }
    }

    pub fn set(&mut self, key: &str, value: &T)
    {
        self.handle.set(key, value).unwrap();
    }

    pub fn get(&self, key: &str) -> Option<T>
    {
        (&self.handle.get(key)).clone()
    }
}

use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref USERDB: Mutex<Database<User>> = Mutex::new(Database::<User>::new("user.db"));
}

pub fn get_user(name: &str) -> Option<User>
{
    let db = USERDB.lock().unwrap();
    db.get(name)
}

pub fn add_user(username: &str, password: &str)
{
    let mut db = USERDB.lock().unwrap();
    db.set(username, &User::new(username, password));
}
