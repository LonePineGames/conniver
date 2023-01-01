
use std::collections::HashMap;

use crate::{val::{Val, p_all}, exec::{State, Stackframe}};

pub type Callback = fn(args: Vec<Val>, &mut State);

pub fn get_builtins() -> HashMap<String, Val> {
  let mut builtins = HashMap::new();

  builtins.insert("quote".to_string(), Val::Builtin(true, quote_cb));
  builtins.insert("lambda".to_string(), Val::Builtin(true, lambda_cb));
  builtins.insert("define".to_string(), Val::Builtin(true, define_cb));
  builtins.insert("if".to_string(), Val::Builtin(true, if_cb));

  builtins.insert("load".to_string(), Val::Builtin(false, load_cb));
  builtins.insert("car".to_string(), Val::Builtin(false, car_cb));
  builtins.insert("cdr".to_string(), Val::Builtin(false, cdr_cb));
  builtins.insert("cons".to_string(), Val::Builtin(false, cons_cb));
  builtins.insert("do".to_string(), Val::Builtin(false, do_cb));
  builtins.insert("+".to_string(), Val::Builtin(false, plus_cb));
  builtins.insert("-".to_string(), Val::Builtin(false, minus_cb));
  builtins.insert("*".to_string(), Val::Builtin(false, mult_cb));
  builtins.insert("/".to_string(), Val::Builtin(false, div_cb));
  builtins.insert("%".to_string(), Val::Builtin(false, modulo_cb));
  builtins.insert("=".to_string(), Val::Builtin(false, eq_cb));
  builtins.insert("<".to_string(), Val::Builtin(false, less_cb));
  builtins.insert(">".to_string(), Val::Builtin(false, greater_cb));
  builtins.insert("<=".to_string(), Val::Builtin(false, less_eq_cb));
  builtins.insert(">=".to_string(), Val::Builtin(false, greater_eq_cb));
  builtins.insert("list?".to_string(), Val::Builtin(false, type_list_cb));
  builtins.insert("symbol?".to_string(), Val::Builtin(false, type_sym_cb));
  builtins.insert("number?".to_string(), Val::Builtin(false, type_num_cb));
  builtins.insert("lambda?".to_string(), Val::Builtin(false, type_lambda_cb));
  builtins.insert("not".to_string(), Val::Builtin(false, not_cb));
  builtins.insert("event".to_string(), Val::Builtin(false, event_cb));
  builtins.insert("apply".to_string(), Val::Builtin(false, apply_cb));
  builtins.insert("format".to_string(), Val::Builtin(false, format_cb));
  builtins.insert("set-program".to_string(), Val::Builtin(false, set_program_cb));

  builtins
}

fn quote_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  } else {
    state.return_stackframe(args[0].clone());
  }
}

fn lambda_cb(args: Vec<Val>, state: &mut State) {
  let list = vec![Val::Sym("lambda".to_string())];
  let list = list.into_iter().chain(args.into_iter()).collect();
  state.return_stackframe(Val::List(list));
}

fn define_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  }

  let mut val = if args.len() > 1 {
    args[1].clone()
  } else {
    Val::nil()
  };

  match &args[0] {

    Val::Sym(sym) => {
      let val = match &val {
        Val::List(list) => {
          if list.is_empty() {
            Val::nil()
          } else {
            let frame = state.get_stackframe();
            if frame.pc <= 2 {
              frame.pc = 2;
              state.add_stackframe(list.clone());
              return;
            } else {
              val.clone()
            }
          }
        },

        Val::Sym(sym) => {
          if let Some(val) = state.get_var(&sym) {
            val.clone()
          } else {
            val.clone()
          }
        },

        _ => {
          val.clone()
        },
      };

      state.set_var(sym.to_string(), val.clone());
    },

    Val::List(calllist) => {
      // lambda
      if calllist.is_empty() {
        // val = Val::List(vec![
        //   Val::Sym("lambda".to_string()),
        //   Val::List(vec![]),
        //   val.clone(),
        // ]);
      } else {
        val = Val::List(vec![
          Val::Sym("lambda".to_string()),
          Val::List(calllist[1..].to_vec()),
          val.clone(),
        ]);

        let first = calllist[0].clone();
        if let Val::Sym(sym) = first {
          state.set_var(sym, val.clone());
        }
      }
    }

    _ => {
      // do nothing
    },
  }

  state.return_stackframe(Val::nil());
}

fn load_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  }

  let val = if let Val::Sym(sym) = &args[0] {
    sym
  } else {
    state.print_error(format!("Could not load file: {:?}", args[0]));
    state.return_stackframe(Val::nil());
    return;
  };

  state.print(format!("Loading file: {}", val));

  let file = std::fs::read_to_string(val);
  if let Ok(file) = file {
    let val = p_all(&file);
    let frame = state.get_stackframe();
    frame.accum = val.clone();
    frame.init = val;
    frame.pc = 0;
  } else {
    state.print_error(format!("Could not load file: {}", val));
    state.return_stackframe(Val::nil());
  }
}

fn car_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  } else if let Val::List(list) = &args[0] {
    if list.is_empty() {
      state.return_stackframe(Val::nil());
    } else {
      state.return_stackframe(list[0].clone());
    }
  } else {
    state.return_stackframe(Val::nil());
  }
}

fn cdr_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  } else if let Val::List(list) = &args[0] {
    if list.is_empty() {
      state.return_stackframe(Val::nil());
    } else {
      state.return_stackframe(Val::List(list[1..].to_vec()));
    }
  } else {
    state.return_stackframe(Val::nil());
  }
}

fn cons_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  } else if let Val::List(list) = &args[1] {
    let mut new_list = vec![args[0].clone()];
    new_list.extend(list.clone());
    state.return_stackframe(Val::List(new_list));
  } else {
    state.return_stackframe(Val::nil());
  }
}

fn do_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
  } else {
    state.return_stackframe(args[args.len() - 1].clone());
  }
}

fn if_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
    return;
  }

  let cond = &args[0];

  let cond = match cond {
    Val::List(list) => {
      if list.is_empty() {
        false
      } else {
        let frame = state.get_stackframe();
        if frame.pc <= 1 {
          frame.pc = 1;
          state.add_stackframe(list.clone());
          return;
        } else {
          list.len() > 0
        }
      }
    },

    Val::Sym(sym) => {
      let val = state.get_var(&sym);
      if let Some(val) = val {
        if val.is_nil() {
          false
        } else {
          true
        }
      } else {
        true
      }
    },

    _ => {
      true
    },
  };

  let val = if cond {
    if args.len() < 2 {
      Val::nil()
    } else {
      args[1].clone()
    }
  } else {
    if args.len() < 3 {
      Val::nil()
    } else {
      args[2].clone()
    }
  };

  match &val {
    Val::List(list) => {
      let frame = state.get_stackframe();
      frame.accum = list.clone();
      frame.init = list.clone();
      frame.pc = 0;
    },

    Val::Sym(sym) => {
      if let Some(val) = state.get_var(&sym) {
        state.return_stackframe(val.clone());
      } else {
        state.return_stackframe(val.clone());
      }
    },

    _ => {
      state.return_stackframe(val);
    },
  }
}

fn plus_cb(args: Vec<Val>, state: &mut State) {
  let mut sum = 0.0;
  for arg in args {
    match arg {
      Val::Num(num) => sum += num,
      _ => {}
    }
  }
  state.return_stackframe(Val::Num(sum));
}

fn minus_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::Num(0.0));
    return;
  }

  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  if args.len() == 1 {
    state.return_stackframe(Val::Num(-sum));
    return;
  }

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => sum -= num,
      _ => {}
    }
  }
  state.return_stackframe(Val::Num(sum));
}

fn mult_cb(args: Vec<Val>, state: &mut State) {
  let mut sum = 1.0;
  for arg in args {
    match arg {
      Val::Num(num) => sum *= num,
      _ => {}
    }
  }
  state.return_stackframe(Val::Num(sum));
}

fn div_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::Num(0.0));
    return;
  }
  
  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  if args.len() == 1 {
    state.return_stackframe(Val::Num(1.0 / sum));
    return;
  }

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => sum /= num,
      _ => {}
    }
  }
  state.return_stackframe(Val::Num(sum));
}

fn modulo_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::Num(0.0));
    return;
  }
  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  if args.len() == 1 {
    state.return_stackframe(Val::Num(sum));
    return;
  }

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => sum %= num,
      _ => {}
    }
  }
  state.return_stackframe(Val::Num(sum));
}

fn eq_cb(args: Vec<Val>, state: &mut State) {
  if args.len() < 2 {
    state.return_stackframe(Val::truth());
    return;
  }

  let val = args[0].clone();
  for arg in args[1..].iter() {
    if val != *arg {
      state.return_stackframe(Val::nil());
      return;
    }
  }
  state.return_stackframe(Val::truth());
}

fn greater_cb(args: Vec<Val>, state: &mut State) {
  if args.len() == 0 {
    state.return_stackframe(Val::lies());
    return;
  } else if args.len() == 1 {
    state.return_stackframe(Val::truth());
    return;
  }

  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => {
        if sum <= *num {
          state.return_stackframe(Val::lies());
          return;
        }
        sum = *num;
      },
      _ => {}
    }
  }
  state.return_stackframe(Val::truth());
}

fn less_cb(args: Vec<Val>, state: &mut State) {
  if args.len() == 0 {
    state.return_stackframe(Val::nil());
    return;
  } else if args.len() == 1 {
    state.return_stackframe(Val::truth());
    return;
  }

  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => {
        if sum >= *num {
          state.return_stackframe(Val::lies());
          return;
        }
        sum = *num;
      },
      _ => {}
    }
  }
  state.return_stackframe(Val::truth());
}

fn greater_eq_cb(args: Vec<Val>, state: &mut State) {
  if args.len() == 0 {
    state.return_stackframe(Val::lies());
    return;
  } else if args.len() == 1 {
    state.return_stackframe(Val::truth());
    return;
  }

  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => {
        if sum < *num {
          state.return_stackframe(Val::lies());
          return;
        }
        sum = *num;
      },
      _ => {}
    }
  }
  state.return_stackframe(Val::truth());
}

fn less_eq_cb(args: Vec<Val>, state: &mut State) {
  if args.len() == 0 {
    state.return_stackframe(Val::lies());
    return;
  } else if args.len() == 1 {
    state.return_stackframe(Val::truth());
    return;
  }

  let mut sum = if let Val::Num(num) = args[0] {
    num
  } else {
    0.0
  };

  for arg in args[1..].iter() {
    match arg {
      Val::Num(num) => {
        if sum > *num {
          state.return_stackframe(Val::lies());
          return;
        }
        sum = *num;
      },
      _ => {}
    }
  }
  state.return_stackframe(Val::truth());
}

fn type_list_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::lies());
  } else if let Val::List(_) = &args[0] {
    state.return_stackframe(Val::truth());
  } else {
    state.return_stackframe(Val::lies());
  }
}

fn type_sym_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::lies());
  } else if let Val::Sym(_) = &args[0] {
    state.return_stackframe(Val::truth());
  } else {
    state.return_stackframe(Val::lies());
  }
}

fn type_num_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::lies());
  } else if let Val::Num(_) = &args[0] {
    state.return_stackframe(Val::truth());
  } else {
    state.return_stackframe(Val::lies());
  }
}

fn type_lambda_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::lies());
  } else if let Val::List(list) = &args[0] {
    if list.is_empty() {
      state.return_stackframe(Val::lies());
    } else if let Val::Sym(sym) = &list[0] {
      if sym == "lambda" {
        state.return_stackframe(Val::truth());
      } else {
        state.return_stackframe(Val::lies());
      }
    } else {
      state.return_stackframe(Val::lies());
    }
  } else {
    state.return_stackframe(Val::lies());
  }
}

fn not_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::truth())
  } else if args[0].is_nil() {
    state.return_stackframe(Val::truth())
  } else {
    state.return_stackframe(Val::lies())
  }
}

fn event_cb(args: Vec<Val>, state: &mut State) {
  state.send_event(Val::List(args));
  state.return_stackframe(Val::nil());
}

fn apply_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
    return;
  }

  let mut list = vec![args[0].clone()];

  if args.len() > 1 {
    if let Val::List(list2) = args[1].clone() {
      list.extend(list2);
    }
  }

  let frame = state.get_stackframe();
  frame.init = list.clone();
  frame.accum = list;
  frame.pc = 0;
}

fn format_cb(args: Vec<Val>, state: &mut State) {
  let mut string = String::new();
  for arg in args.iter() {
    match arg {
      Val::Num(num) => string.push_str(&num.to_string()),
      Val::Sym(sym) => string.push_str(sym),
      Val::List(list) => string.push_str(&format!("{:?}", list)),
      Val::Builtin(_, _) => string.push_str("<builtin>"),
    }
  }
  state.return_stackframe(Val::Sym(string));
}

fn set_program_cb(args: Vec<Val>, state: &mut State) {
  if args.is_empty() {
    state.return_stackframe(Val::nil());
    return;
  }

  state.set_main_program(args[0].clone());
  state.return_stackframe(Val::nil());
}