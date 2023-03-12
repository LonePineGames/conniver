
use rust_lisp::{parser::parse, model::{Value, Symbol}};
use std::fmt::Debug;

use crate::{exec::State, variables::ScopeRef};

#[derive(Clone)]
pub enum Val {
  Sym(String),
  String(String),
  Num(f32),
  List(Vec<Val>),
  Builtin(bool, fn(Vec<Val>, &mut State)),
  Lambda(bool, ScopeRef, Vec<Val>),
  Message(String),
}

impl Val {
  pub fn nil() -> Val {
    Val::List(vec![])
  }

  pub fn truth() -> Val {
    Val::Sym("t".to_string())
  }
  
  pub fn lies() -> Val {
    Val::nil()
  }

  pub fn is_nil(&self) -> bool {
    match self {
      Val::List(list) => list.is_empty(),
      _ => false,
    }
  }

  pub fn is_callable(&self) -> bool {
    match self {
      Val::List(list) => {
        if list.is_empty() {
          false
        } else if let Val::Sym(sym) = &list[0] {
          sym == "lambda"
        } else {
          false
        }
      },
      Val::Builtin(_, _) => true,
      _ => false,
    }
  }

  pub(crate) fn memory_usage(&self) -> usize {
    match self {
      Val::Sym(sym) => sym.len(),
      Val::String(string) => string.len(),
      Val::Num(_) => 4,
      Val::List(list) => {
        let mut size = 4;
        for val in list {
          size += val.memory_usage();
        }
        size
      },
      Val::Builtin(_, _) => 4,
      Val::Lambda(_, _, list) => {
        let mut size = 4;
        for val in list {
          size += val.memory_usage();
        }
        size
      },
      Val::Message(message) => message.len(),
    }
  }
}

impl ToString for Val {
  fn to_string(&self) -> String {
    match self {
      Val::Sym(sym) => {
        sym.to_string()
      },
      Val::String(string) => {
        format!("\"{}\"", string)
      },
      Val::Num(num) => num.to_string(),
      Val::List(list) => {
        let mut s = String::new();
        s.push('(');
        for (i, val) in list.iter().enumerate() {
          if i > 0 {
            s.push(' ');
          }
          s.push_str(&val.to_string());
        }
        s.push(')');
        s
      },
      Val::Builtin(special, _) => {
        if *special {
          "<special>".to_string()
        } else {
          "<builtin>".to_string()
        }
      },
      Val::Lambda(_, vars, list) => {
        let mut s = String::new();
        s.push_str(&format!("<L{}>(", vars.0));
        for (i, val) in list.iter().enumerate() {
          if i > 0 {
            s.push(' ');
          }
          s.push_str(&val.to_string());
        }
        s.push(')');
        s
      },
      Val::Message(message) => {
        message.to_string()
      },
    }
  }
}

impl Default for Val {
  fn default() -> Self {
    Val::nil()
  }
}

impl Debug for Val {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.to_string())
  }
}

impl PartialEq for Val {
  fn eq(&self, other: &Self) -> bool {

    match (self, other) {
      (Val::Sym(sym1), Val::Sym(sym2)) => sym1 == sym2,
      (Val::String(string1), Val::String(string2)) => string1 == string2,
      (Val::Sym(_), Val::String(_)) => false,
      (Val::String(_), Val::Sym(_)) => false,
      (Val::Num(num1), Val::Num(num2)) => num1 == num2,
      (Val::List(list1), Val::List(list2)) => list1 == list2,
      (Val::Lambda(_, _, list1), Val::Lambda(_, _, list2)) => list1 == list2,
      (Val::Lambda(_, _, list1), Val::List(list2)) => list1 == list2,
      (Val::List(list1), Val::Lambda(_, _, list2)) => list1 == list2,
      _ => false,
    }
  }
}

pub fn p(s: &str) -> Val {
  let mut ast_iter = parse(&s);
  let ast = ast_iter.next(); 
  if ast.is_none() {
    let error = format!("Error parsing: {}", s);
    return Val::Sym(error);
  }
  let ast_inner = ast.unwrap();
  if ast_inner.is_err() {
    let error = format!("Error parsing: {}", s);
    return Val::Sym(error);
  }
  let first_expression = ast_inner.unwrap();

  value_to_val(first_expression)
}

pub fn p_all(s: &str) -> Vec<Val> {
  let mut ast_iter = parse(&s);
  let mut result = vec![];
  loop {
    let ast = ast_iter.next(); 
    if ast.is_none() {
      return result;
    }
    let ast_inner = ast.unwrap();
    if ast_inner.is_err() {
      continue;
    }
    let expression = ast_inner.unwrap();
  
    result.push(value_to_val(expression));
  }
}

fn value_to_val(value: Value) -> Val {
  match value {
    Value::Symbol(Symbol(name)) => {
      //catch a bug in the parser
      if name == "\"\"".to_owned() {
        Val::String("".to_string())
      } else {
        Val::Sym(name)
      }
    },
    Value::String(string) => Val::String(string),
    Value::List(list) => Val::List(list.into_iter().map(value_to_val).collect()),
    Value::Int(int) => Val::Num(int as f32),
    Value::Float(float) => Val::Num(float),
    Value::True => Val::Sym("t".to_string()),
    Value::False => Val::Sym("f".to_string()),
    //Value::HashMap(hash_map) => SValue::List(SList { list: hash_map.into_iter().map(|(key, value)| SValue::List(SList { list: vec![value_to_svalue(key), value_to_svalue(value)] })).collect() }),
    //Value::NativeFunction(native_function) => SValue::Symbol(native_function.name.clone()),
    //Value::Lambda(lambda) => SValue::List(SList { list: vec![SValue::Symbol("lambda".to_string()), SValue::List(SList { list: lambda.args.into_iter().map(value_to_svalue).collect() }), value_to_svalue(lambda.body)] }),
    Value::Lambda(_) => Val::Sym("lambda".to_string()),
    Value::HashMap(_) => Val::Sym("hashmap".to_string()),
    Value::NativeFunc(_) => Val::Sym("nativefunc".to_string()),
    Value::Macro(_) => Val::Sym("macro".to_string()),
    Value::Foreign(_) => Val::Sym("foreign".to_string()),
    Value::TailCall {func: _, args: _} => Val::Sym("tailcall".to_string()),
  }
}
