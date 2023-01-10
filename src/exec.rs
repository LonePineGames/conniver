use std::{fmt::{Formatter, Debug}};

use crate::{val::{Val, p, p_all}, builtins::get_builtins, screen::{ScreenLine, ScreenColor}, variables::{VarSpace, VarRef}};

#[derive(Clone)]
pub struct Stackframe {
  pub vars: VarRef,
  pub init: Vec<Val>,
  pub accum: Vec<Val>,
  pub pc: usize,
}

impl Debug for Stackframe {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "Stackframe {{\n  init: {:?},\n  accum: {:?},\n  pc: {:?}  }}", Val::List(self.init.clone()), Val::List(self.accum.clone()), self.pc)
  }
}

#[derive(Clone)]
pub struct State {
  pub vars: VarSpace,
  pub stack: Vec<Stackframe>,
  pub back_stack: Vec<Vec<Stackframe>>,
  pub result: Val,
  events: Vec<Val>,
}

impl State {
  pub fn new() -> State {
    let mut vars = VarSpace::new();
    let root = vars.root();
    vars.set_all(root, get_builtins());
    State { 
      vars,
      result: Val::nil(),
      stack: vec![],
      back_stack: vec![],
      events: vec![],
    }
  }

  pub fn load_lib(&mut self) {
    println!("Loading library...");
    eval_s(&p("(load \"cnvr/lib.cnvr\")"), self);
    let lib = include_bytes!("../cnvr/lib.cnvr");
    let val = p_all(&String::from_utf8_lossy(lib));
    self.add_stackframe(val);
    self.run();
  }

  pub fn get_var_ref(&self) -> VarRef {
    if self.stack.is_empty() {
      self.vars.root()
    } else {
      let frame = self.stack.last().unwrap();
      frame.vars
    }
  }

  pub fn get_var(&self, name: &String) -> Option<&Val> {
    let var_ref = self.get_var_ref();
    self.vars.get(var_ref, name)
  }

  pub fn set_var(&mut self, name: &String, val: Val) {
    let var_ref = self.get_var_ref();
    let var_ref = self.vars.parent(var_ref);
    let var_ref = self.vars.parent(var_ref);
    self.vars.set(var_ref, name, val);
  }

  pub fn debug_state(&self) -> Vec<ScreenLine> {
    let mut result = vec![];
    for frame in self.stack.iter() {
      for (i, val) in frame.init.iter().enumerate() {
        let text = if i == 0 {
          format!("({:?} ...)", val)
        } else {
          format!("{:?}", val)
        };
        let indent = if i == 0 { 0 } else { 2 };
        let indent = " ".repeat(indent);
        let color = if i == frame.pc { ScreenColor::Green } else { ScreenColor::White };
        let importance = 20 - 2*(i as i32 - frame.pc as i32).abs();

        result.push(ScreenLine { indent, text, color, order: 0, importance });
      }
    }

    // make sure order is correct
    let result = result.iter().enumerate().map(|(i, line)| -> ScreenLine {
      let line = line.clone();
      ScreenLine { order: i as i32, ..line }
    }).collect::<Vec<_>>();

    return result;
  }

  pub fn add_stackframe(&mut self, list: Vec<Val>) {
    println!("Adding stackframe: {:?}", list);
    let var_ref = self.get_var_ref();
    let vars = self.vars.new_child(var_ref);

    self.stack.push(Stackframe {
      vars,
      init: list.clone(),
      accum: list,
      pc: 0,
    });
  }

  pub fn return_stackframe(&mut self, val: Val) {
    println!("Returning stackframe: {:?}", val);
    if !self.stack.is_empty() {
      self.stack.pop();
    }
    if self.stack.is_empty() {
      self.result = val;
    } else {
      let frame = self.stack.last_mut().unwrap();
      frame.accum[frame.pc] = val;
      frame.pc += 1;
    }
  }

  pub fn replace_stackframe(&mut self, val: Vec<Val>) {
    println!("Replacing stackframe: {:?}", val);
    let frame = self.stack.last_mut().unwrap();
    frame.accum = val.clone();
    frame.init = val;
    frame.pc = 0;
  }

  pub fn call(&mut self) {
    println!("Calling...");
    if self.stack.is_empty() {
      return;
    }

    let frame = self.stack.last_mut().unwrap();
    if frame.accum.is_empty() {
      self.return_stackframe(Val::nil());
    } else {
      let callable = frame.accum[0].clone();
      let callable = if let Val::Macro(val) = callable {
        Val::List(val)
      } else {
        callable
      };
      if let Val::Builtin(_, callback) = callable {
        let args = frame.accum[1..].to_vec();
        callback(args, self);

      } else if let Val::List(list) = callable {
        if list.len() < 3 || list[0] != Val::Sym("lambda".to_string()) {
          let result = Val::List(frame.accum.clone());
          self.return_stackframe(result);
        } else {

          let root = self.vars.root();
          let var_ref = self.vars.new_child(root);
          let params = list[1].clone();
          match params {
            Val::List(params) => {
              for (i, param) in params.iter().enumerate() {
                if let Val::Sym(sym) = param {
                  if i + 1 < frame.accum.len() {
                    self.vars.set(var_ref, &sym.to_string(), frame.accum[i + 1].clone());
                  }
                }
              }
            },

            Val::Sym(sym) => {
              self.vars.set(var_ref, &sym.to_string(), Val::List(frame.accum[1..].to_vec()));
            },

            _ => {},
          }

          if list.len() > 3 {
            let frame = self.get_stackframe();
            frame.accum = vec![Val::Sym("do".to_string())];
            frame.accum.extend(list[2..].iter().cloned());
            frame.init = frame.accum.clone();
            frame.vars = var_ref;
            frame.pc = 0;

          } else {
            match &list[2] {
              Val::List(list) => {
                frame.vars = var_ref;
                frame.init = list.clone();
                frame.accum = list.clone();
                frame.pc = 0;
              },

              Val::Sym(sym) => {
                if let Some(val) = self.vars.get(var_ref, sym) {
                  self.return_stackframe(val.clone());
                } else {
                  self.return_stackframe(list[2].clone());
                }
              },

              _ => {
                self.return_stackframe(list[2].clone());
              },
            }
          }
        }
      } else {
        let result = Val::List(frame.accum.clone());
        self.return_stackframe(result);
      }
    }
  }

  pub fn step(&mut self) -> Option<Val> {
    if !self.stack.is_empty() {
      self.step_inner();
    }

    if self.stack.is_empty() {
      if self.back_stack.is_empty() {
        return Some(self.result.clone());
      } else {
        self.stack = self.back_stack.pop().unwrap();
        return None;
      }
    } else {
      return None;
    }
  }

  fn step_inner(&mut self) {
    let frame = self.stack.last_mut().unwrap();
    if frame.pc >= frame.accum.len() {
      self.call();
      return;
    }

    if frame.pc >= 1 {
      if let Val::Builtin(true, callback) = frame.accum[0] {
        let args = frame.accum[1..].to_vec();
        callback(args, self);
        return;
      } else if let Val::Macro(_) = frame.accum[0] {
        self.call();
        return;
      }
    }

    let val = frame.accum[frame.pc].clone();

    match val {
      Val::Sym(sym) => {
        if let Some(val) = self.vars.get(frame.vars, &sym) {
          frame.accum[frame.pc] = val.clone();
        }
        frame.pc += 1;
      },

      Val::List(list) => {
        self.add_stackframe(list);
      },

      _ => {
        frame.pc += 1;
      },
    };
  }

  pub fn set_program(&mut self, val: Val) {
    self.stack.clear();

    match &val {
      Val::List(list) => {
        self.add_stackframe(list.clone());
      },

      Val::Sym(sym) => {
        if let Some(val) = self.get_var(&sym) {
          self.result = val.clone();
        } else {
          self.result = val.clone();
        }
      },

      _ => {
        self.result = val.clone();
      },
    }
  }

  pub fn set_main_program(&mut self, prog: Val) {
    let prog = match prog {
      Val::List(list) => list,
      _ => vec![prog],
    };

    let root = self.vars.root();
    let vars = self.vars.new_child(root);

    let stack = vec![Stackframe {
      init: prog.clone(),
      accum: prog,
      pc: 0,
      vars,
    }];
    
    if self.back_stack.is_empty() {
      self.back_stack.push(stack);
    } else {
      self.back_stack[0] = stack;
    }
  }

  pub fn get_stackframe(&mut self) -> &mut Stackframe {
    self.stack.last_mut().unwrap()
  }

  pub fn running(&self) -> bool {
    (!self.stack.is_empty() || !self.back_stack.is_empty()) && self.events.is_empty()
  }

  pub fn run(&mut self) -> Option<Val> {
    while self.running() {
      if let Some(result) = self.step() {
        return Some(result);
      }
    }
    None
  }

  pub fn process_events(&mut self) {
    for event in &self.events {
      if let Val::List(list) = event {
        if !list.is_empty() {
          if let Val::Sym(sym) = &list[0] {
            let sym: &str = sym;
            match sym {
              "print" => {
                if list.len() > 1 {
                  if let Val::Sym(str) = &list[1] {
                    println!("{}", str);
                  } else {
                    println!("{:?}", list[1]);
                  }
                }
              },
              _ => {
                println!("Unknown event: {:?}", event);
              },
            }
          }
        }
      }
    }

    self.events.clear();
  }

  pub fn take_event(&mut self) -> Option<Val> {
    if self.events.is_empty() {
      None
    } else {
      Some(self.events.remove(0))
    }
  }

  pub fn send_event(&mut self, event: Val) {
    self.events.push(event);
  }

  pub fn print(&mut self, err: String) {
    self.send_event(p(&format!("(print \"{}\")", err)));
  }

  pub fn print_error(&mut self, err: String) {
    self.send_event(p(&format!("(print \"{}\")", err)));
  }

  pub fn interrupt(&mut self, val: Val) {
    self.back_stack.push(self.stack.clone());
    self.set_program(val);
  }
}

pub fn eval(val: Val) -> Val {
  let mut state = State::new();
  eval_s(&val, &mut state)
}

pub fn eval_s(val: &Val, state: &mut State) -> Val {
  state.set_program(val.clone());
  for _ in 0..10000 {
    state.process_events();
    if let Some(val) = state.step() {
      return val;
    }
  }
  Val::Sym("Error: Too many steps".to_string())
}

/*
pub fn eval_sc(val: &Val, state: &mut State, context: &Context) -> Val {
  match &val {
    Val::Num(_) => val.clone(),

    Val::Sym(sym) => {
      if let Some(var) = state.get_var(sym) {
        var.clone()
      } else {
        val.clone()
      }
    },

    Val::List(list) => {
      if list.is_empty() {
        Val::nil()
      } else {
        let first = list[0].clone();
        if let Val::Sym(sym) = first {
          call(sym, list[1..].to_vec(), state, context)
        } else {
          let list: Vec<Val> = list.into_iter().map(|arg| eval_sc(&arg, state, context)).collect();
          Val::List(list)
        }
      }
    }
  }
}

pub fn call(sym: String, args: Vec<Val>, state: &mut State, context: &Context) -> Val {
  let special_form = context.special_forms.get(&sym);
  if let Some(callback) = special_form {
    callback(args, state, context)
  } else {
    let args: Vec<Val> = args.into_iter().map(|arg| eval_sc(&arg, state, context)).collect();
    
    if let Some(var) = state.get_var(&sym) {
      match &var {
        Val::Num(num) => {
          let list = vec![Val::Num(*num)];
          let list = list.into_iter().chain(args.into_iter()).collect();
          return Val::List(list);
        },

        Val::Sym(sym) => {
          let list = vec![Val::Sym(sym.to_string())];
          let list = list.into_iter().chain(args.into_iter()).collect();
          return Val::List(list);
        },

        Val::List(elems) => {
          if elems.len() >= 3 && elems[0] == Val::Sym("lambda".to_string()) {
            let mut new_state = state.clone();
            if let Val::List(params) = &elems[1] {
              for (param, arg) in params.iter().zip(args.iter()) {
                if let Val::Sym(sym) = param {
                  new_state.set_var(sym, arg.clone());
                }
              }
            } else if let Val::Sym(sym) = &elems[1] {
              let sym = sym.to_string();
              if args.is_empty() {
                new_state.set_var(&sym, Val::nil());
              } else {
                new_state.set_var(&sym, args[0].clone());
              }
            }
            eval_sc(&elems[2], &mut new_state, context)

          } else {
            
            let list = vec![var.clone()];
            let list = list.into_iter().chain(args.into_iter()).collect();
            return Val::List(list);
          }
        }
      }
      
    } else if let Some(callback) = context.callbacks.get(&sym) {
      callback(args, state, context)

    } else {
      let err_message = format!("Unknown procedure: {}", sym);
      Val::Sym(err_message)
    }
  }
}*/
