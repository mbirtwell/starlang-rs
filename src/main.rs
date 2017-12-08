extern crate lalrpop_util;
#[macro_use]
extern crate indoc;

pub mod ast;
pub mod grammar;
mod comment_stripper;

#[cfg(test)]
mod test_grammar;

pub mod exec_tree;

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
