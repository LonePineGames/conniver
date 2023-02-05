use crate::{val::*, exec::{eval, State, eval_s}, object::{read_ivec2, read_object, read_string}};

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
  assert_eq!(eval(p("(string? \"a\")")), p("t"));
  assert_eq!(eval(p("(string? 'a)")), p("()"));
  assert_eq!(eval(p("(string? a)")), p("()"));
  assert_eq!(eval(p("(number? 5)")), p("t"));
  assert_eq!(eval(p("(number? a)")), p("()"));
  assert_eq!(eval(p("(lambda? a)")), p("()"));
  assert_eq!(eval(p("(lambda? 5)")), p("()"));

  assert_ne!(p("\"a\""), p("a"));
  
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
  s.load_lib();
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
  s.set_program(p("(print (list 1 2 3))"));
  s.run();
  assert_eq!(s.take_event(), Some(p("(print \"(1 2 3)\")")));
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

#[test]
fn test_message() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  s.message_add("test");
  s.set_program(p("(test 1 2 3)"));
  assert_eq!(s.message_peek(), None);
  s.run();
  assert_eq!(s.message_peek(), Some(vec![p("test"), p("1"), p("2"), p("3")]));

  s.set_program(p("(+ 2 (test 4 5 6))"));
  s.run();
  assert_eq!(s.message_peek(), Some(vec![p("test"), p("4"), p("5"), p("6")]));
  s.message_return(p("7"));
  assert_eq!(s.message_peek(), None);
  s.run();
  assert_eq!(s.message_peek(), None);
  assert_eq!(s.result, p("9"));
}

#[test]
fn test_read_object() {
  let obj = p("((name test) (type item) (size 1 1) (health 100) (speed 0.1) (range 0) (power 0) (consumes 0) (outputs 0) (requirements 0) (buildtime 0) (description \"This is some complex object\"))");

  let mut props = 0;
  read_object(&obj, |key, val| {
    props += 1;
    match key {
      "name" => assert_eq!(val, &p("test")),
      "type" => assert_eq!(val, &p("item")),
      "size" => {
        assert_eq!(val, &p("(1 1)"));
        read_ivec2(&val, |x, y| {
          assert_eq!(x, 1);
          assert_eq!(y, 1);
        }, || panic!("Invalid size"));
      },
      "health" => {
        assert_eq!(val, &p("100"));
        read_ivec2(&val, |_, _| {
          panic!("read_ivec2 should not be called on a single value");
        }, || ());
      },
      "speed" => assert_eq!(val, &p("0.1")),
      "range" => assert_eq!(val, &p("0")),
      "power" => assert_eq!(val, &p("0")),
      "consumes" => assert_eq!(val, &p("0")),
      "outputs" => assert_eq!(val, &p("0")),
      "requirements" => assert_eq!(val, &p("0")),
      "buildtime" => assert_eq!(val, &p("0")),
      "description" => {
        assert_eq!(val, &p("\"This is some complex object\""));
        assert_eq!(read_string(val), "This is some complex object");
      },
      _ => panic!("Unknown key: {}", key),
    }
  });
  assert_eq!(props, 12);
}