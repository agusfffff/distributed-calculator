//! Módulo que define operaciones aritméticas y su parsing desde strings.
use std::{str::FromStr};

#[derive(PartialEq, Eq, Debug)]

/// Operaciones soportadas por la calculadora
pub enum Operation {
    /// Suma de un valor `u8`
    Add(u8),
    /// Resta de un valor `u8`    
    Sub(u8),
    /// Multiplicación por un valor `u8`
    Mul(u8),
    /// División por un valor `u8` (no permite dividir por cero)
    Div(u8),
}

impl FromStr for Operation {
    type Err = String;
    /// Convierte un string en una operación 
    /// 
    /// # Formato esperado 
    /// <operaor> <valor>
    /// 
    /// Operadores válidos: `+`, `-`, `*`, `/`.
    ///     
    /// # Ejemplo
    /// let op = Operation::from_str("+ 10").unwrap();
    /// 
    /// # Errores
    /// - Si el string no tiene exactamente 2 tokens → `"expected 2 arguments"`.
    /// - Si el segundo token no es un número válido → `"parsing error: invalid integer"`.
    /// - División por cero → `"division by zero"`.
    /// - Operador desconocido → `"parsing error: unknown operation"`.
    /// 
    fn from_str(tokens: &str) -> Result<Self, Self::Err> {
        let vector: Vec<&str> = tokens.split_whitespace().collect();

        let [operation, operand] = vector.try_into().map_err(|_| "expected 2 arguments")?;

        let operand = operand.parse().map_err(|e| format!("parsing error: invalid integer: {}", e))?;

        match operation {
            "+" => Ok(Operation::Add(operand)),
            "-" => Ok(Operation::Sub(operand)),
            "*" => Ok(Operation::Mul(operand)),
            "/" => { if operand == 0 {
                        Err("division by zero".to_string())
                    } else {
                        Ok(Operation::Div(operand))
                }
            },
            _ => Err(format!("parsing error: unknown operation: {}", operation)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Operation;
    use std::str::FromStr;

    #[test]
    fn test_correct_parsing() {
        assert_eq!(Operation::from_str("+ 10"), Ok(Operation::Add(10)));
        assert_eq!(Operation::from_str("- 20"), Ok(Operation::Sub(20)));
        assert_eq!(Operation::from_str("* 30"), Ok(Operation::Mul(30)));
        assert_eq!(Operation::from_str("/ 40"), Ok(Operation::Div(40)));
    }

    #[test]
    fn test_incorrect_quantity_of_arguments() {
        assert_eq!(Operation::from_str("+"), Err("expected 2 arguments".to_string()));
        assert_eq!(Operation::from_str("+ 10 20"), Err("expected 2 arguments".to_string()));
    }

    #[test]
    fn test_not_an_integer_operand() {
        assert_eq!(Operation::from_str("+ ten"), Err("parsing error: invalid integer: invalid digit found in string".to_string()));
    }

    #[test]
    fn test_too_large_integer() {
        assert_eq!(Operation::from_str("+ 300"), Err("parsing error: invalid integer: number too large to fit in target type".to_string()));
    }

    #[test]
    fn test_unknown_operation() {
        assert_eq!(Operation::from_str("% 10"), Err("parsing error: unknown operation: %".to_string()));
    }

    #[test]
    fn test_division_by_zero() {
        assert_eq!(Operation::from_str("/ 0"), Err("division by zero".to_string()));
    }
}