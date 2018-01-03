use std::fmt;
use std::fmt::Write;
use rand::Rng;

use rvs_parser::ast;
use model::{Expr, ExprData};

#[derive(Clone)]
pub struct Binary {
    data: ExprData,
    operation: ast::BinaryOpcode,
    l: Box<Expr>,
    r: Box<Expr>,
}

#[derive(Clone)]
pub struct Unary {
    data: ExprData,
    operation: ast::UnaryOpcode,
    operand: Box<Expr>,
}

impl Binary {
    pub fn new(l: Box<Expr>, operation: ast::BinaryOpcode, r: Box<Expr>) -> Binary {
        Binary {
            data: ExprData {
                prev: 0,
                done: false,
            },
            operation: operation,
            l: l,
            r: r,
        }
    }
}

impl Expr for Binary {
    fn next(&mut self, rng: &mut Rng) -> u32 {
        let (l, r) = (self.l.next(rng), self.r.next(rng));

        self.data.done = self.l.done() || self.r.done();

        self.data.prev = match self.operation {
            ast::BinaryOpcode::Or => l | r,
            ast::BinaryOpcode::Xor => l ^ r,
            ast::BinaryOpcode::And => l & r,
            ast::BinaryOpcode::Shl => l << r,
            ast::BinaryOpcode::Shr => l >> r,
            ast::BinaryOpcode::Add => l + r,
            ast::BinaryOpcode::Sub => l - r,
            ast::BinaryOpcode::Mul => l * r,
            ast::BinaryOpcode::Div => l / r,
            ast::BinaryOpcode::Mod => l % r,
        };

        self.data.prev
    }

    fn data(&self) -> &ExprData {
        &self.data
    }
}

impl fmt::Display for Binary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_char('(')?;
        self.l.fmt(f)?;
        f.write_char(' ')?;
        self.operation.fmt(f)?;
        f.write_char(' ')?;
        self.r.fmt(f)?;
        f.write_char(')')
    }
}

impl Unary {
    pub fn new(operation: ast::UnaryOpcode, operand: Box<Expr>) -> Unary {
        Unary {
            data: ExprData {
                prev: 0,
                done: false,
            },
            operation,
            operand,
        }
    }
}

impl Expr for Unary {
    fn next(&mut self, rng: &mut Rng) -> u32 {
        let operand = self.operand.next(rng);

        self.data.done = self.operand.done();

        self.data.prev = match self.operation {
            ast::UnaryOpcode::Neg => !operand,
        };

        self.data.prev
    }

    fn data(&self) -> &ExprData {
        &self.data
    }
}

impl fmt::Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.operation.fmt(f)?;
        self.operand.fmt(f)
    }
}
