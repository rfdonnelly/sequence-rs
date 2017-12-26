extern crate rvs_parser;

mod utils;
use utils::*;

#[test]
fn good() {
    assert_eq!(
        parse("a = (5);"),
        "[Assignment(Identifier(\"a\"), Number(5))]");
    assert_eq!(
        parse("a = 5;"),
        "[Assignment(Identifier(\"a\"), Number(5))]");
    assert_eq!(
        parse("a = 0xa;"),
        "[Assignment(Identifier(\"a\"), Number(10))]");
    assert_eq!(
        parse("a = 0xaf;"),
        "[Assignment(Identifier(\"a\"), Number(175))]");
}

#[test]
fn bad() {
    assert!(parse_result("a = (5));").is_err());
    assert!(parse_result("a = (5;").is_err());
}

#[test]
fn operations() {
    assert_eq!(
        parse("a = 1+2;"),
        "[Assignment(Identifier(\"a\"), BinaryOperation(Number(1), Add, Number(2)))]");
    assert_eq!(
        parse("a = 1+2*3;"),
        "[Assignment(Identifier(\"a\"), BinaryOperation(Number(1), Add, BinaryOperation(Number(2), Mul, Number(3))))]");
}
