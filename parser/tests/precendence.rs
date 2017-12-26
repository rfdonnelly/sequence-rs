extern crate rvs_parser;

fn parse(s: &str) -> String {
    format!("{:?}", rvs_parser::parse(s, &mut rvs_parser::SearchPath::new()).unwrap())
}

mod precedence {
    use super::*;

    #[test]
    fn unary() {
        assert_eq!(
            parse("a = ~0 + 1;"),
            "[Assignment(Identifier(\"a\"), BinaryOperation(UnaryOperation(Neg, Number(0)), Add, Number(1)))]");
    }
}