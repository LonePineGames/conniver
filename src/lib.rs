#![crate_type = "lib"]

pub mod builtins;
pub mod exec;
pub mod object;
pub mod val;
pub mod variables;

#[cfg(test)]
pub mod test;

pub use crate::val::Val;
pub use crate::val::p;
pub use crate::exec::{eval, State, eval_s};
pub use crate::object::{read_object, read_ivec2};

