extern crate rvs;

use std::collections::HashSet;

use rvs::Context;

#[test]
fn reverse() {
    let mut context = Context::new();
    rvs::parse("a = [1, 0];", &mut context).unwrap();
    rvs::transform(&mut context).unwrap();

    let a = context.get("a").unwrap();

    let expected: HashSet<u32> =
        [0, 1].iter().cloned().collect();
    let mut actual: HashSet<u32> = HashSet::new();

    for _ in 0..10 {
        actual.insert(a.borrow_mut().next());
    }

    assert_eq!(expected, actual);
}

#[test]
fn same() {
    let mut context = Context::new();
    rvs::parse("a = [1, 1];", &mut context).unwrap();
    rvs::transform(&mut context).unwrap();

    let a = context.get("a").unwrap();

    for _ in 0..10 {
        assert_eq!(a.borrow_mut().next(), 1);
    }
}