#![crate_type = "lib"]

pub mod builtins;
pub mod exec;
pub mod screen;
pub mod val;

#[cfg(test)]
pub mod test;

pub use crate::val::Val;
pub use crate::val::p;
pub use crate::exec::{eval, State, eval_s};

