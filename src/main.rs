extern crate lalrpop_util;

pub mod ast;
pub mod grammar;

#[cfg(test)]
mod test_grammar;

mod exec_tree;

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
