mod config;
mod pfollow;
mod database;
mod message;
mod zmqtest;

fn main() {
    println!("Hello, world!");
    pfollow::test_actable()
}
