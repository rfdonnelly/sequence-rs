use std::collections::HashMap;

mod util;
use crate::util::*;

#[test]
fn yields_each_value_once_per_cycle() {
    let a = expr_to_var("{1, 2, 4, 8}").unwrap();
    let mut a = a.borrow_mut();

    let mut actual: HashMap<u32, u32> = HashMap::new();
    for i in 1..25 {
        let expected: HashMap<u32, u32> =
            [(1, i), (2, i), (4, i), (8, i)].iter().cloned().collect();

        for _ in 0..4 {
            let entry = actual.entry(a.next()).or_insert(0);
            *entry += 1;
        }

        assert_eq!(expected, actual);
    }
}

#[test]
fn selects_another_subexpr_when_current_subexpr_done() {
    let a = expr_to_var("{Pattern(0, 1), Pattern(2, 3)}").unwrap();
    let mut a = a.borrow_mut();

    for _ in 0..100 {
        let value = a.next();

        match value {
            0 => {
                assert_eq!(a.next(), 1);
                assert_eq!(a.next(), 2);
                assert_eq!(a.next(), 3);
            }
            2 => {
                assert_eq!(a.next(), 3);
                assert_eq!(a.next(), 0);
                assert_eq!(a.next(), 1);
            }
            _ => {
                panic!("unreachable");
            }
        }
    }
}

fn test_done(expr: &str) {
    let a = expr_to_var(expr).unwrap();
    let mut a = a.borrow_mut();

    assert_eq!(a.done(), false);

    let expected: Vec<(u32, bool)> = (0..4)
        .zip(vec![false, false, false, true].into_iter())
        .cycle()
        .take(16)
        .map(|(_, done)| (0, done))
        .collect();
    let actual: Vec<(u32, bool)> = (0..16).map(|_| (a.next(), a.done())).collect();

    assert_eq!(actual, expected);
}

#[test]
fn done_when_all_done() {
    test_done("{Pattern(0, 0), Pattern(0, 0)}");
}

#[test]
fn done_when_all_done2() {
    test_done("{0, 0, 0, 0}");
}
