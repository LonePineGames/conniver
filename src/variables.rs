use std::collections::HashMap;

use crate::val::Val;

#[derive(Clone, Copy, Debug)]
pub struct VarRef(usize);

#[derive(Clone, Debug)]
pub struct Vars {
  vars: HashMap<String, Val>,
  parent: VarRef,
}

#[derive(Clone, Debug)]
pub struct VarSpace {
  vars: Vec<Vars>,
}

impl VarSpace {
  pub fn new() -> VarSpace {
    VarSpace {
      vars: vec![Vars {
        vars: HashMap::new(),
        parent: VarRef(0),
      }],
    }
  }

  pub fn root(&self) -> VarRef {
    VarRef(0)
  }

  pub(crate) fn parent(&self, var_ref: VarRef) -> VarRef {
    self.vars[var_ref.0].parent
  }

  pub fn new_child(&mut self, parent: VarRef) -> VarRef {
    let new = VarRef(self.vars.len());
    self.vars.push(Vars {
      vars: HashMap::new(),
      parent,
    });
    new
  }

  pub fn get(&self, scope: VarRef, var: &str) -> Option<&Val> {
    let mut s = scope;
    loop {
      if let Some(val) = self.vars[s.0].vars.get(var) {
        return Some(val);
      }
      if s.0 == 0 {
        return None;
      }
      s = self.vars[s.0].parent;
    }
  }

  pub fn set(&mut self, scope: VarRef, var: &str, val: Val) {
    self.vars[scope.0].vars.insert(var.to_string(), val);
  }

  pub fn set_all(&mut self, scope: VarRef, vars: HashMap<String, Val>) {
    self.vars[scope.0].vars.extend(vars);
  }
}