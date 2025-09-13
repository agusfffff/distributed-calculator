use std::ops::{Add, Div, Mul, Sub};

use crate::operation::Operation;

#[derive(Default)]
pub struct Calculator {
    accumulation: i32,
}

impl Calculator {
    pub fn new() -> Self {
        Self { accumulation: 0 }
    }

    pub fn _accumulation(&self) -> i32 {
        self.accumulation
    }

    pub fn apply(&mut self, op: Operation) {
        match op {
            Operation::Add(operand) => self.accumulation = self.accumulation.add(operand),
            Operation::Sub(operand) => self.accumulation = self.accumulation.sub(operand),
            Operation::Mul(operand) => self.accumulation = self.accumulation.mul(operand),
            Operation::Div(operand) => self.accumulation = self.accumulation.div(operand),
        }
    }
}
