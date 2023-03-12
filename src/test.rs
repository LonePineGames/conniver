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
  
  // variable pollution
  assert_eq!(eval_s(&p("a"), s), p("a"));
  eval_s(&p("(define mutate (lambda (x) (define a x) a))"), s);
  assert_eq!(eval_s(&p("(mutate 10)"), s), p("10"));
  assert_eq!(eval_s(&p("a"), s), p("a"));
  println!("{}", s.describe());

  eval_s(&p("(define f (lambda (x) x))"), s);
  assert_eq!(eval_s(&p("(f 10)"), s), p("10"));
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
  // s.load_lib();

  eval_s(&p("(define (add x) (lambda (y) (+ x y)))"), s);
  assert_eq!(eval_s(&p("((add 1) 2)"), s), p("3"));
  assert_eq!(eval_s(&p("((add 2) 3)"), s), p("5"));
  assert_eq!(eval_s(&p("((add 3) 4)"), s), p("7"));

  eval_s(&p("(define (mul x) (lambda (y) (* x y)))"), s);
  assert_eq!(eval_s(&p("((mul 2) 3)"), s), p("6"));
  assert_eq!(eval_s(&p("((mul ((add 1) 2)) 3)"), s), p("9"));
  assert_eq!(eval_s(&p("((mul ((add 1) 2)) ((add 1) 2))"), s), p("9"));
  assert_eq!(eval_s(&p("((mul ((mul 4) 2)) ((add 1) 2))"), s), p("24"));

  eval_s(&p("(define add5 (add 5))"), s);
  assert_eq!(eval_s(&p("(add5 10)"), s), p("15"));
  assert_eq!(eval_s(&p("(add5 ((mul 2) 3))"), s), p("11"));
  assert_eq!(eval_s(&p("((mul 2) (add5 3))"), s), p("16"));

  eval_s(&p("(define (future x) (lambda () x))"), s);
  assert_eq!(eval_s(&p("((future 4))"), s), p("4"));
  eval_s(&p("(define five (future 5))"), s);
  println!("{}", s.describe());
  assert_eq!(eval_s(&p("(five)"), s), p("5"));
  assert_eq!(eval_s(&p("((future 6))"), s), p("6"));
  assert_eq!(eval_s(&p("(five)"), s), p("5"));
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
fn test_message_loop() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  s.message_add("test");
  s.set_program(p("(loop (test 1 2 3) (test 4 5 6))"));

  for i in 0..10 {
    println!("iteration {}", i);
    assert_eq!(s.message_peek(), None);
    s.run();
    assert_eq!(s.message_peek(), Some(vec![p("test"), p("1"), p("2"), p("3")]));
    s.message_return(p("7"));
    assert_eq!(s.message_peek(), None);
    s.run();
    assert_eq!(s.message_peek(), Some(vec![p("test"), p("4"), p("5"), p("6")]));
    s.message_return(p("8"));
    assert_eq!(s.message_peek(), None);

    //check tail recursion
    println!("{}", s.describe());
    assert_eq!(s.stack.len(), 1);
  }
}

#[test]
fn test_read_object() {
  let obj = p("((name test) (type item) (size 1 1) (health 100) (speed 0.1) (range 0) (power 0) (consumes 0) (outputs 0) (requirements 0) (buildtime 0) (very-cool) (description \"This is some complex object\"))");

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
      "very-cool" => assert_eq!(val, &p("()")),
      "description" => {
        assert_eq!(val, &p("\"This is some complex object\""));
        assert_eq!(read_string(val), "This is some complex object");
      },
      _ => panic!("Unknown key: {}", key),
    }
  });
  assert_eq!(props, 13);
}

#[test]
fn test_memory_management() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  let init_mem = s.memory_usage();
  eval_s(&p("(define (fib n) (if (< n 2) n (+ (fib (- n 1)) (fib (- n 2)))))"), s);
  assert_eq!(s.memory_usage(), init_mem + 74);
  assert_eq!(eval_s(&p("(fib 10)"), s), p("55"));
  assert_eq!(s.stack.len(), 0);
  assert_eq!(s.memory_usage(), init_mem + 74);

  eval_s(&p("(define (add x) (lambda (y) (+ x y)))"), s);
  eval_s(&p("(define add5 (add 5))"), s);
  eval_s(&p("(define (mul x) (lambda (y) (* x y)))"), s);
  let init_mem = s.memory_usage();
  assert_eq!(eval_s(&p("((add 1) 2)"), s), p("3"));
  assert_eq!(eval_s(&p("((add 2) 3)"), s), p("5"));
  assert_eq!(eval_s(&p("((add 3) 4)"), s), p("7"));
  assert_eq!(eval_s(&p("((mul 2) 3)"), s), p("6"));
  assert_eq!(eval_s(&p("((mul ((add 1) 2)) 3)"), s), p("9"));
  assert_eq!(eval_s(&p("((mul ((add 1) 2)) ((add 1) 2))"), s), p("9"));
  assert_eq!(eval_s(&p("((mul ((mul 4) 2)) ((add 1) 2))"), s), p("24"));
  assert_eq!(eval_s(&p("(add5 10)"), s), p("15"));
  assert_eq!(eval_s(&p("(add5 ((mul 2) 3))"), s), p("11"));
  assert_eq!(eval_s(&p("((mul 2) (add5 3))"), s), p("16"));
  assert_eq!(s.stack.len(), 0);
  //assert_eq!(s.vars.memory_usage(), init_mem);
  assert_eq!(s.memory_usage(), init_mem + 182);
}

#[test]
fn test_string() {
  let mut state = State::new();
  let s = &mut state;
  s.load_lib();

  assert_eq!(eval_s(&p("(string? \"\")"), s), p("t"));
  assert_eq!(eval_s(&p("(string? \"a\")"), s), p("t"));
  assert_eq!(eval_s(&p("(string? abc)"), s), p("()"));

  assert_eq!(eval_s(&p("(string-length \"\")"), s), p("0"));
  assert_eq!(eval_s(&p("(string-length \"a\")"), s), p("1"));
  assert_eq!(eval_s(&p("(string-length \"abc\")"), s), p("3"));

  assert_eq!(eval_s(&p("(string-cons \"\" \"\")"), s), p("\"\""));
  assert_eq!(eval_s(&p("(string-cons \"a\" \"\")"), s), p("\"a\""));
  assert_eq!(eval_s(&p("(string-cons \"\" \"a\")"), s), p("\"a\""));
  assert_eq!(eval_s(&p("(string-cons \"a\" \"b\")"), s), p("\"ab\""));
  assert_eq!(eval_s(&p("(string-cons \"a\" \"b\" \"c\")"), s), p("\"abc\""));
  assert_eq!(eval_s(&p("(string-cons \"a\" \"b\" \"c\" \"d\")"), s), p("\"abcd\""));
  
  assert_eq!(eval_s(&p("(string-head \"\")"), s), p("\"\""));
  assert_eq!(eval_s(&p("(string-head \"a\")"), s), p("\"a\""));
  assert_eq!(eval_s(&p("(string-head \"ab\")"), s), p("\"a\""));
  assert_eq!(eval_s(&p("(string-head \"abc\")"), s), p("\"a\""));

  assert_eq!(eval_s(&p("(string-tail \"\")"), s), p("\"\""));
  assert_eq!(eval_s(&p("(string-tail \"a\")"), s), p("\"\""));
  assert_eq!(eval_s(&p("(string-tail \"ab\")"), s), p("\"b\""));
  assert_eq!(eval_s(&p("(string-tail \"abc\")"), s), p("\"bc\""));
}

