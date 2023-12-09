use serde_encrypt::shared_key::SharedKey;
use lazy_static::lazy_static;

pub const IDENTITY_FILE: &str = "./cert/peach.pfx";

pub const SECRET: &str = "peach";

lazy_static! {
    pub static ref KEY: SharedKey = SharedKey::new("peachpeachpeachpeachpeachpeachpe".as_bytes().try_into().unwrap());
}
