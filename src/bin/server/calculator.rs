//! Representa una calculadora simple que mantiene una acumulación y puede aplicar
//! operaciones aritméticas básicas (suma, resta, multiplicación y división) a esa  
//! acumulación.    
//!     

use crate::operation::Operation;

#[derive(Default)]
pub struct Calculator {
    /// La acumulación actual de la calculadora.
    accumulation: u8,
}

impl Calculator {
    /// Crea una nueva instancia de Calculator con la acumulación inicial en 0.
    pub fn new() -> Self {
        Self { accumulation: 0 }
    }

    /// Devuelve el valor actual de la acumulación.
    pub fn accumulation(&self) -> u8 {
        self.accumulation
    }

    /// Aplica una operación a la acumulación actual.
    /// La operación puede ser suma, resta, multiplicación o división.
    pub fn apply(&mut self, op: Operation) {
        match op {
            Operation::Add(operand) => self.accumulation = self.accumulation.wrapping_add(operand),
            Operation::Sub(operand) => self.accumulation = self.accumulation.wrapping_sub(operand),
            Operation::Mul(operand) => self.accumulation = self.accumulation.wrapping_mul(operand),
            Operation::Div(operand) => self.accumulation = self.accumulation.wrapping_div(operand),
        }
    }
}

#[cfg(test)]
mod tests { 
    use super::Calculator;
    use crate::operation::Operation;

    #[test]
    fn test_add(){ 
        let mut calc = Calculator::new();
        calc.apply(Operation::Add(10));
        assert_eq!(calc.accumulation(), 10);
    }

    #[test]
    fn test_substract(){ 
        let mut calc = Calculator::new();
        calc.apply(Operation::Add(10));
        calc.apply(Operation::Sub(5));
        assert_eq!(calc.accumulation(), 10 - 5 );
    }

    #[test]
    fn test_multiply(){ 
        let mut calc = Calculator::new();
        calc.apply(Operation::Add(10));
        calc.apply(Operation::Mul(5));
        assert_eq!(calc.accumulation(), 10 * 5 );
    }

    #[test]
    fn test_divide(){ 
        let mut calc = Calculator::new();
        calc.apply(Operation::Add(10));
        calc.apply(Operation::Div(2));
        assert_eq!(calc.accumulation(), 10 / 2 );
    }


} 