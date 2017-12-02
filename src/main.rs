extern crate lalrpop_util;

pub mod ast;
pub mod grammar;

#[cfg(test)]
mod test_grammar;

mod pure_exec_tree;

#[cfg(not(test))]
fn main() {
    println!("Hello, world!");
}
