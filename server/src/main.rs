mod db;
use crate::db::*;

use std::thread;

fn main()
{
    let a = thread::spawn(|| {
        add_user("peach", "peach");
    });
    let b = thread::spawn(|| {
        add_user("papaya", "papaya");
    });
    let c = thread::spawn(|| {
        add_user("apple", "apple");
    });
    a.join().unwrap();
    b.join().unwrap();
    c.join().unwrap();
    println!("{:?}", get_user("apple"));
    println!("{:?}", get_user("peach"));
    println!("{:?}", get_user("papaya"));
}
