extern crate penum;
use penum::penum;

#[penum[tuple(_)]]
enum Must<'a> {
    Static { name: &'a str, age: usize },
}

fn main() {}
