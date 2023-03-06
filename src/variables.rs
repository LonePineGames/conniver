use std::collections::HashMap;

use crate::val::Val;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScopeRef(pub usize);

#[derive(Clone, Debug)]
pub struct Scope {
  vars: HashMap<String, Val>,
  parent: ScopeRef,
}

#[derive(Clone, Debug)]
pub struct VarSpace {
  scopes: Vec<Scope>,
  free_scopes: Vec<ScopeRef>,
}

impl VarSpace {
  pub fn new() -> VarSpace {
    VarSpace {
      scopes: vec![Scope {
        vars: HashMap::new(),
        parent: ScopeRef(0),
      }],
      free_scopes: vec![],
    }
  }

  pub fn root(&self) -> ScopeRef {
    ScopeRef(0)
  }

  // pub(crate) fn parent(&self, var_ref: ScopeRef) -> ScopeRef {
  //   self.scopes[var_ref.0].parent
  // }

  pub fn new_child(&mut self, parent: ScopeRef) -> ScopeRef {
    if self.free_scopes.len() > 0 {
      let new = self.free_scopes.pop().unwrap();
      self.scopes[new.0].parent = parent;
      return new;
    }

    let new = ScopeRef(self.scopes.len());
    self.scopes.push(Scope {
      vars: HashMap::new(),
      parent,
    });
    new
  }

  pub fn get(&self, scope: ScopeRef, var: &str) -> Option<&Val> {
    let mut s = scope;
    loop {
      if let Some(val) = self.scopes[s.0].vars.get(var) {
        return Some(val);
      }
      if s.0 == 0 {
        return None;
      }
      s = self.scopes[s.0].parent;
    }
  }

  pub fn set(&mut self, scope: ScopeRef, var: &str, val: Val) {
    self.scopes[scope.0].vars.insert(var.to_string(), val);
  }

  pub fn set_all(&mut self, scope: ScopeRef, vars: HashMap<String, Val>) {
    self.scopes[scope.0].vars.extend(vars);
  }

  pub fn describe(&self) -> String {
    let mut s = String::new();
    for (i, v) in self.scopes.iter().enumerate() {
      s.push_str(&format!("{}: -> {}\n", i, v.parent.0));
      for (k, v) in v.vars.iter() {
        if let Val::Builtin(_, _) = v {
          continue;
        }
        s.push_str(&format!("  {} = {:?}\n", k, v));
      }
    }
    s
  }

  pub fn memory_usage(&self) -> usize {
    let mut size = std::mem::size_of::<ScopeRef>() * (self.scopes.len() - self.free_scopes.len());
    for v in self.scopes.iter() {
      for (k, v) in v.vars.iter() {
        size += k.capacity() * std::mem::size_of::<u8>();
        size += v.memory_usage();
      }
    }
    size
  }

  pub fn val_has_ancestor(&self, scope: ScopeRef, val: &Val) -> bool {
    let mut list = None;
    match val {
      Val::Lambda(_, var_ref, llist) => {
        if self.scope_has_ancestor(scope, *var_ref) {
          return true;
        }
        list = Some(llist);
      },
      Val::List(llist) => {
        list = Some(llist);
      },
      _ => {},
    }

    if let Some(list) = list {
      for val in list {
        if self.val_has_ancestor(scope, val) {
          return true;
        }
      }
    }

    false
  }

  fn scope_has_ancestor(&self, scope: ScopeRef, ancestor: ScopeRef) -> bool {
    let mut s = scope;
    loop {
      if s == ancestor {
        return true;
      }
      if s.0 == 0 {
        return false;
      }
      let parent = self.scopes[s.0].parent;
      if s == parent {
        return false;
      }
      s = parent;
    }
  }

  pub fn remove_inner(&mut self, scope: ScopeRef) {
    let mut s = &mut self.scopes[scope.0];
    s.vars.clear();
    s.parent = ScopeRef(0);
    self.free_scopes.push(scope);
  }

  pub fn remove(&mut self, scope: ScopeRef) {
    self.remove_inner(scope);
    let to_remove = (0..self.scopes.len())
      .map(|i| ScopeRef(i))
      .filter(|i| *i != scope && self.scope_has_ancestor(*i, scope))
      .collect::<Vec<_>>();
    for i in to_remove {
      self.remove_inner(i);
    }
  }
}