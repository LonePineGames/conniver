use crate::val::Val;

pub fn read_object(object: &Val, mut f: impl FnMut(&str, &Val)) {
  match object {
    Val::List(list) => {
      for item in list {
        match item {
          Val::List(list) => {
            if list.len() == 0 {
              continue;
            }
            let key = match &list[0] {
              Val::Sym(key) => key,
              _ => panic!("Invalid object property key"),
            };
            match list.len() {
              1 => f(key, &Val::nil()),
              2 => f(key, &list[1]),
              _ => f(key, &Val::List(list[1..].to_vec())),
            };
          },
          _ => panic!("Invalid object property"),
        }
      }
    },
    _ => panic!("Invalid object"),
  }
}

pub fn read_ivec2(object: &Val, success: impl FnOnce(i32, i32), failure: impl FnOnce()) {
  if let Val::List(list) = object {
    if list.len() >= 2 {
      if let (Val::Num(x), Val::Num(y)) = (&list[0], &list[1]) {
        success(*x as i32, *y as i32);
        return;
      }
    }
  }
  failure();
}

pub fn read_string(object: &Val) -> String {
  match object {
    Val::Sym(sym) => sym.to_string(),
    Val::String(str) => str.to_string(),
    _ => format!("{:?}", object)
  }
}
