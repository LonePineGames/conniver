use crate::{val::*, exec::{eval, State, eval_s}, screen::{ScreenLine, ScreenColor}};

#[test]
fn test_parsing() {
  assert_eq!(p("(1)"), p("(1)"));
  assert_ne!(p("(1)"), p("(2)"));
  assert_ne!(p("(1)"), p("(1 2)"));
  assert_eq!(p("'(1)"), p("(quote (1))"));
  assert_eq!(eval(p("'(+ 2 2)")), p("(+ 2 2)"));
}

#[test]
fn test_arithmetic() {
  assert_eq!(eval(p("(+ 1 2)")), p("3"));
  assert_eq!(eval(p("(+ 1 2 3)")), p("6"));
  assert_eq!(eval(p("(* 1 2)")), p("2"));
  assert_eq!(eval(p("(/ 1 2)")), p("0.5"));
  assert_eq!(eval(p("(- 1 2)")), p("-1"));
  assert_eq!(eval(p("(- 1)")), p("-1"));
  assert_eq!(eval(p("(/ 4)")), p("0.25"));
  assert_eq!(eval(p("(+ 39 48 72 23 91)")), p("273"));
  assert_eq!(eval(p("(/ 273 5)")), p("54.6"));
  assert_eq!(eval(p("(% 17 12)")), p("5"));
}

#[test]
fn test_arithmetic_nested() {
  assert_eq!(eval(p("(* (+ 1 39) (- 53 45))")), p("320"));
  assert_eq!(eval(p("(/ (+ 39 48 72 23 91) 5)")), p("54.6"));
}

#[test]
fn test_car_cdr() {
  assert_eq!(eval(p("(car '(1 2 3))")), p("1"));
  assert_eq!(eval(p("(cdr '(1 2 3))")), p("(2 3)"));
  assert_eq!(eval(p("(cdr (cons 3 (cons 2 (cons 1 '()))))")), p("(2 1)"));
}

#[test]
fn test_variables() {
  let mut state = State::new();
  let s = &mut state;

  eval_s(&p("(define x 1)"), s);
  assert_eq!(eval_s(&p("x"), s), p("1"));
  
  eval_s(&p("(define x (+ 2 2))"), s);
  assert_eq!(eval_s(&p("x"), s), p("4"));
  
  eval_s(&p("(define y x)"), s);
  assert_eq!(eval_s(&p("y"), s), p("4"));
}

#[test]
fn test_lambda() {
  let mut state = State::new();
  let s = &mut state;

  eval_s(&p("(define hello (lambda () \"Hello World\"))"), s);
  assert_eq!(eval_s(&p("(hello)"), s), p("\"Hello World\""));
  assert_eq!(eval_s(&p("hello"), s), p("(lambda () \"Hello World\")"));

  eval_s(&p("(define greeting \"Hello World\")"), s);
  eval_s(&p("(define hello (lambda () greeting))"), s);
  assert_eq!(eval_s(&p("(hello)"), s), p("\"Hello World\""));

  eval_s(&p("(define hello (lambda () (event print greeting) greeting))"), s);
  eval_s(&p("(define greeting \"Hello Again\")"), s);
  s.set_program(p("(hello)"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(print \"Hello Again\")")));
  assert_eq!(s.run(), Some(p("\"Hello Again\"")));

  eval_s(&p("(define square (lambda (x) (* x x)))"), s);
  assert_eq!(eval_s(&p("(square 30)"), s), p("900"));

  eval_s(&p("(define sum3 (lambda (a b c) (+ a b c)))"), s);
  assert_eq!(eval_s(&p("(sum3 10 20 30)"), s), p("60"));

  eval_s(&p("(define (mult3 a b c) (* a b c)))"), s);
  assert_eq!(eval_s(&p("(+ (mult3 10 30 30) 1)"), s), p("9001"));
}

#[test]
fn test_if() {
  let mut state = State::new();
  let s = &mut state;

  assert_eq!(eval_s(&p("(if (= 1 1) \"yes\" \"no\")"), s), p("\"yes\""));
  assert_eq!(eval_s(&p("(if (> 1 2) \"I'll eat my hat!\" \"no\")"), s), p("\"no\""));
  assert_eq!(eval_s(&p("(if (< 1 2 3) \"yes\" \"no\")"), s), p("\"yes\""));
  assert_eq!(eval_s(&p("(if (> 1 2 3) \"yes\" \"no\")"), s), p("\"no\""));
  
  eval_s(&p("(define x 5)"), s);
  assert_eq!(eval_s(&p("(if (> 2 x) \"yes\" \"no\")"), s), p("\"no\""));
}

#[test]
fn test_types() {
  assert_eq!(eval(p("(list? ())")), p("t"));
  assert_eq!(eval(p("(list? 'a)")), p("()"));
  assert_eq!(eval(p("(symbol? 'a)")), p("t"));
  assert_eq!(eval(p("(symbol? a)")), p("t"));
  assert_eq!(eval(p("(symbol? ())")), p("()"));
  assert_eq!(eval(p("(number? 5)")), p("t"));
  assert_eq!(eval(p("(number? a)")), p("()"));
  assert_eq!(eval(p("(lambda? a)")), p("()"));
  assert_eq!(eval(p("(lambda? 5)")), p("()"));
  
  let mut state = State::new();
  let s = &mut state;
  eval_s(&p("(define x 5)"), s);
  assert_eq!(eval_s(&p("(if (number? x) \"yes\" \"no\")"), s), p("\"yes\""));

  eval_s(&p("(define (hello) \"Hello World\")"), s);
  assert_eq!(eval_s(&p("(if (lambda? hello) \"yes\" \"no\")"), s), p("\"yes\""));
  assert_eq!(eval_s(&p("(if (not (lambda? x)) \"yes\" \"no\")"), s), p("\"yes\""));
}

#[test]
fn test_recursion() {
  let mut state = State::new();
  let s = &mut state;

  eval_s(&p("(define (factorial n) (if (= n 1) 1 (* n (factorial (- n 1)))))"), s);
  assert_eq!(eval_s(&p("(factorial 5)"), s), p("120"));

  eval_s(&p("(define (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))"), s);
  assert_eq!(eval_s(&p("(fib 10)"), s), p("55"));

  eval_s(&p("(define (list*2 ls) 
    (if ls
      (cons (* 2 (car ls)) (list*2 (cdr ls))
      '())))"), s);
  assert_eq!(eval_s(&p("(list*2 '(1 2 3 4 5))"), s), p("(2 4 6 8 10)"));
}

#[test]
fn test_print() {
  let mut state = State::new();
  let s = &mut state;
  eval_s(&p("(define (print x) (event 'print x))"), s);
  s.set_program(p("(print \"Hello World\")"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(print \"Hello World\")")));
  
  eval_s(&p("(define print (lambda x (event 'print (apply format x))))"), s);
  s.set_program(p("(print \"Hello World\")"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(print \"Hello World\")")));
  s.set_program(p("(print \"Hello World No. \" 2 \"!\")"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(print \"Hello World No. 2!\")")));
}

#[test]
pub fn test_collatz() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  eval_s(&p("(define collatz (lambda (n) 
    (print n)
    (if (= n 1) 
      1
      (if (even? n) 
        (+ 1 (collatz (/ n 2)))
        (+ 1 (collatz (+ (* 3 n) 1)))))))"), s);

  assert_eq!(eval_s(&p("(collatz 10)"), s), p("7"));
  assert_eq!(eval_s(&p("(collatz 100)"), s), p("26"));
  assert_eq!(eval_s(&p("(collatz 1161)"), s), p("182"));
}

#[test]
pub fn test_debug() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  eval_s(&p("(load \"cnvr/velocity.cnvr\")"), s);
  assert_eq!(s.take_event(), None);
  s.set_program(p("(loop (goto 20 20) (pick ingot autoprocessor) (goto 20 30) (place pad))"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(goto 20 20)")));

  let debug = s.debug_state();
  assert_eq!(debug.len(), 5);
  assert_eq!(debug[0], ScreenLine { indent: "".to_string(), text: "(loop ...)".to_string(), 
      color: ScreenColor::White, order: 0, importance: 16 });
  assert_eq!(debug[1], ScreenLine { indent: "  ".to_string(), text: "(goto 20 20)".to_string(), 
      color: ScreenColor::White, order: 1, importance: 18 });
  assert_eq!(debug[2], ScreenLine { indent: "  ".to_string(), text: "(pick ingot autoprocessor)".to_string(), color: ScreenColor::Green, order: 2, importance: 20 });
  assert_eq!(debug[3], ScreenLine { indent: "  ".to_string(), text: "(goto 20 30)".to_string(), 
      color: ScreenColor::White, order: 3, importance: 18 });
  assert_eq!(debug[4], ScreenLine { indent: "  ".to_string(), text: "(place pad)".to_string(), 
      color: ScreenColor::White, order: 4, importance: 16 });
}
 
#[test]
fn test_interrupt() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  eval_s(&p("(load \"cnvr/velocity.cnvr\")"), s);
  assert_eq!(s.take_event(), None);
  s.set_program(p("(loop (goto 20 20) (pick ingot autoprocessor) (goto 20 30) (place pad))"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(goto 20 20)")));

  s.interrupt(p("(input-key a)"));
  assert_eq!(s.take_event(), None);
  s.run();
  assert_eq!(s.take_event(), Some(p("(move w)")));
}

#[test]
fn test_cond() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  assert_eq!(eval_s(&p("(cond ((= 1 1) 2) (else 3))"), s), p("2"));
  eval_s(&p("(define (m x) (cond ((= x 1) 1) ((= x 2) 2) (else 3)))"), s);
  assert_eq!(eval_s(&p("(m 1)"), s), p("1"));
  assert_eq!(eval_s(&p("(m 2)"), s), p("2"));
  assert_eq!(eval_s(&p("(m 4)"), s), p("3"));
}

#[test]
fn test_closure() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  eval_s(&p("(define (add x) (lambda (y) (+ x y)))"), s);
  assert_eq!(eval_s(&p("((add 1) 2)"), s), p("3"));
  assert_eq!(eval_s(&p("((add 2) 3)"), s), p("5"));
  assert_eq!(eval_s(&p("((add 3) 4)"), s), p("7"));
}
