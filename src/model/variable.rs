use super::expr::Expr;
use crate::transform::CrateRng;

use std::fmt;
use std::rc::{Rc, Weak};
use std::cell::RefCell;

pub struct Variable {
    expr: Box<dyn Expr>,
    rng: CrateRng,
}

pub type VariableRef = Rc<RefCell<Box<Variable>>>;
pub type VariableWeak = Weak<RefCell<Box<Variable>>>;

impl Variable {
    pub fn new(expr: Box<dyn Expr>, rng: CrateRng) -> Variable {
        Variable { expr, rng }
    }

    pub fn clone_expr(&self) -> Box<dyn Expr> {
        self.expr.clone()
    }

    #[cfg_attr(feature = "cargo-clippy", allow(should_implement_trait))]
    pub fn next(&mut self) -> u32 {
        self.expr.next(&mut self.rng)
    }

    pub fn prev(&self) -> u32 {
        self.expr.prev()
    }

    pub fn done(&self) -> bool {
        self.expr.done()
    }
}

impl fmt::Display for Variable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.expr.fmt(f)
    }
}
