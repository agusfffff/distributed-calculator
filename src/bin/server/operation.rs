use std::str::FromStr;

#[derive(PartialEq, Eq, Debug)]
pub enum Operation {
    Add(i32),
    Sub(i32),
    Mul(i32),
    Div(i32),
}

impl FromStr for Operation {
    type Err = &'static str;

    fn from_str(tokens: &str) -> Result<Self, Self::Err> {
        // Try to convert the vector into a statically-sized array of 2 elements, failing otherwise.
        let vector: Vec<&str> = tokens.split_whitespace().collect();

        let [operation, operand] = vector.try_into().map_err(|_| "expected 2 arguments")?;

        // Parse the operand into an u8.
        let operand = operand.parse().map_err(|_| "operand is not an i32")?;

        match operation {
            "+" => Ok(Operation::Add(operand)),
            "-" => Ok(Operation::Sub(operand)),
            "*" => Ok(Operation::Mul(operand)),
            "/" => Ok(Operation::Div(operand)),
            _ => Err("unknown operation"),
        }
    }
}
