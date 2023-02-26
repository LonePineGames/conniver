use std::io::{self, Write};

use exec::State;
use crate::val::p;

pub mod builtins;
pub mod exec;
pub mod object;
pub mod val;
pub mod variables;

fn main() {
  let mut state = State::new();
  let mut input = String::new();
  loop {
    input.clear();
    print!("> ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();
    state.set_program(p(&input));
    while state.running() {
      state.run();
    }
    println!("{:?}", state.result);
  }
}
