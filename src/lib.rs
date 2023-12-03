use std::process::exit;

use anyhow::{anyhow, Result};

use token::{Token, TokenStream};

mod token;

fn expression(ts: &mut TokenStream, variables: &mut VarTable) -> Result<f64> {
    let mut value = term(ts, variables)?;

    loop {
        match ts.peek()? {
            Some(Token::Symbol('+')) => {
                ts.next()?;
                value += term(ts, variables)?;
            }
            Some(Token::Symbol('-')) => {
                ts.next()?;
                value -= term(ts, variables)?;
            }
            _ => break
        }
    }

    Ok(value)
}

fn term(ts: &mut TokenStream, variables: &mut VarTable) -> Result<f64> {
    let mut value = primary(ts, variables)?;

    loop {
        match ts.peek()? {
            Some(Token::Symbol('*')) => {
                ts.next()?;
                value *= primary(ts, variables)?;
            }
            Some(Token::Symbol('/')) => {
                ts.next()?;
                value /= primary(ts, variables)?;
            }
            _ => break
        }
    }

    Ok(value)
}

fn primary(ts: &mut TokenStream, variables: &mut VarTable) -> Result<f64> {
    match ts.next()? {
        Some(Token::Number(n)) => Ok(n),
        Some(Token::Symbol('(')) => {
            let value = expression(ts, variables)?;
            match ts.next()? {
                Some(Token::Symbol(')')) => Ok(value),
                _ => anyhow::bail!("Expected closing parenthesis")
            }
        }
        Some(Token::Symbol('-')) => {
            Ok(-primary(ts, variables)?)
        }
        Some(Token::Symbol('+')) => {
            Ok(primary(ts, variables)?)
        }
        Some(Token::Name(name)) => {
            if let Some(value) = variables.retrieve(&name) {
                Ok(value)
            } else {
                anyhow::bail!("Undefined variable: {}", name)
            }
        }
        _ => anyhow::bail!("Expected a number, a variable or an opening parenthesis")
    }
}

fn statement(ts: &mut TokenStream, variables: &mut VarTable) -> Result<f64> {
    match ts.peek()? {
        Some(Token::Let) => {
            ts.next().expect("Should be a let token");

            let label = if let Some(Token::Name(name)) = ts.next()? {
                name
            } else {
                anyhow::bail!("A name is expected after let")
            };

            if variables.contains(&label) {
                anyhow::bail!("Variable {} is already defined", label)
            }

            if ts.next()?.is_some_and(|token| token != Token::Symbol('=')) {
                anyhow::bail!("Expected an = token after let {}", label)
            }

            let value = expression(ts, variables)?;

            variables.store(&label, value);

            Ok(value)
        }
        Some(Token::Name(label)) => {
            ts.next().expect("Should be a name token");

            if let Some(Token::Symbol('=')) = ts.peek()? {
                ts.next().expect("Should be an = token");

                if !variables.contains(&label) {
                    anyhow::bail!("Variable {} is not defined. Use let to define it before assigning a value. Example: 'let {} = 5; x'", label, label)
                }

                let value = expression(ts, variables)?;
                variables.store(&label, value);
                Ok(value)
            } else {
                let val = variables.retrieve(&label).ok_or_else(|| anyhow!("Undefined variable: {}", label))?;
                ts.put_back(Token::Number(val)); // Now the variable value occupies the same place as it's label did
                expression(ts, variables)
            }
        }
        _ => expression(ts, variables)
    }
}

#[derive(Debug, PartialEq)]
enum EvaluationResult {
    Number(f64),
    Error(String),
    Quit,
}

fn evaluate(expression: &str, variables: &mut VarTable) -> Vec<EvaluationResult> {
    let mut ts = TokenStream::new(expression.as_bytes());
    let mut val: Option<f64> = None;
    let mut res = vec![];

    loop {
        let token = ts.peek()
            .unwrap_or_else(|e| {
                res.push(EvaluationResult::Error(format!("Error while reading input: {}", e)));
                ts.discard_invalid();
                Some(Token::Noop)
            });

        match token {
            Some(Token::Noop) => {}
            Some(Token::EndStatement) => {
                ts.next().expect("Should have an end statement token in the stream");
                if let Some(val) = val { res.push(EvaluationResult::Number(val)); }
                val = None;
            }
            Some(Token::Quit) => {
                res.push(EvaluationResult::Quit);
                ts.next().expect("Should have a quit token in the stream");
            }
            Some(token) => {
                statement(&mut ts, variables)
                    .map(|result| res.push(EvaluationResult::Number(result)))
                    .unwrap_or_else(|e| {
                        res.push(EvaluationResult::Error(format!("Error occurred while evaluating token of type {}: {}", token, e)));
                        ts.discard_invalid();
                    });
            }
            None => {
                if let Some(val) = val { res.push(EvaluationResult::Number(val)) }
                break;
            }
        }
    }

    res
}

struct Variable {
    label: String,
    value: f64,
}

struct VarTable(Vec<Variable>);

impl VarTable {
    fn store(&mut self, label: &String, value: f64) {
        for var in self.0.iter_mut() {
            if *var.label == *label {
                var.value = value;
                return;
            }
        }

        self.0.push(Variable { label: label.clone(), value });
    }

    fn contains(&self, label: &String) -> bool {
        for var in self.0.iter() {
            if var.label == *label { return true; }
        }

        return false;
    }

    fn retrieve(&self, label: &String) -> Option<f64> {
        for var in self.0.iter() {
            if *var.label == *label { return Some(var.value); }
        }

        None
    }
}

fn prompt() -> String {
    inquire::Text::new("")
        .prompt()
        .expect("Failed to read input")
}

pub fn calculate() {
    let mut input = String::new();
    let mut should_quit = false;
    let mut variables: VarTable = VarTable(vec![]);

    loop {
        input.clear();
        input.push_str(prompt().trim());

        for result in evaluate(input.as_str(), &mut variables) {
            match result {
                EvaluationResult::Number(n) => println!("={}", n),
                EvaluationResult::Error(e) => println!("Error: {}", e),
                EvaluationResult::Quit => should_quit = true
            }
        }

        if should_quit { exit(0) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evaluate_with_spaces() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5 + 3", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(8.0)]);
    }

    #[test]
    fn test_evaluate_without_spaces() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5+3", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(8.0)]);
    }

    #[test]
    fn test_evaluate_with_multiple_spaces() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5   +   3", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(8.0)]);
    }

    #[test]
    fn test_evaluate_with_invalid_expression() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5 + * 3", &mut variables);
        assert!(
            matches!(result[0], EvaluationResult::Error(_)),
        )
    }

    #[test]
    fn test_evaluate_with_variable() {
        let mut variables = VarTable(vec![Variable { label: "x".to_string(), value: 5.0 }]);
        let result = evaluate("x + 3", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(8.0)]);
    }

    #[test]
    fn test_evaluate_with_undefined_variable() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("x + 3", &mut variables);
        for r in result.iter() {
            assert!(
                matches!(r, EvaluationResult::Error(_)),
                "Result should be an error for expression 'x + 3' when x is undefined, but was {:?}", r
            )
        }
    }

    #[test]
    fn test_evaluate_with_assignment() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let x = 5", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(5.0)], "Result should be 5 for expression 'let x = 5'");
        assert_eq!(variables.retrieve(&"x".to_string()), Some(5.0), "Variable x should be 5 after assignment");
    }

    #[test]
    fn test_evaluate_with_invalid_assignment() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let 5 = x", &mut variables);
        for r in result.iter() {
            assert!(
                matches!(r, EvaluationResult::Error(_)),
                "Result should be an error for expression 'let 5 = x'"
            )
        }
    }

    #[test]
    fn test_evaluate_with_multiple_operations() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5 + 3 * 2", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(11.0)]);
    }

    #[test]
    fn test_evaluate_with_parentheses() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("(5 + 3) * 2", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(16.0)]);
    }

    #[test]
    fn test_evaluate_with_unbalanced_parentheses() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("(5 + 3 * 2", &mut variables);
        assert_eq!(result.len(), 1, "Result should contain only one element");
        assert!(
            matches!(result[0], EvaluationResult::Error(_)),
            "Result should be an error for expression '(5 + 3 * 2'"
        )
    }

    #[test]
    fn test_evaluate_empty_input() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("", &mut variables);
        assert_eq!(result, Vec::new(), "Result should be an empty vector for empty input");
    }

    #[test]
    fn test_evaluate_with_negative_numbers() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("-5 + 3", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(-2.0)], "Negative number calculation should work");
    }

    #[test]
    fn test_evaluate_with_decimal_numbers() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("2.5 * 4", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(10.0)], "Decimal number calculation should work");
    }

    #[test]
    fn test_evaluate_with_complex_expression() {
        let mut variables = VarTable(vec![Variable { label: "y".to_string(), value: 2.0 }]);
        let result = evaluate("3 * (2 + y) / 2", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(6.0)], "Complex expression with variable should be evaluated correctly");
    }

    #[test]
    fn test_evaluate_with_mixed_operations() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("2 + 3 * 4 - 5 / 2", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(11.5)], "Expression with mixed operations should be evaluated correctly");
    }

    #[test]
    fn test_evaluate_with_whitespace_input() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("    ", &mut variables);
        assert_eq!(result, Vec::new(), "Result should be an empty vector for input with only whitespace");
    }

    #[test]
    fn test_evaluate_with_trailing_semicolon() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5 + 3;", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(8.0)], "Trailing semicolon should be ignored");
    }

    #[test]
    fn test_evaluate_with_multiple_semicolons() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("5 + 3 ;; 2 * 4", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(8.0), EvaluationResult::Number(8.0)], "Multiple consecutive semicolons should be handled correctly");
    }

    #[test]
    fn test_evaluate_with_let_and_variable() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let x = 5; x + 3", &mut variables);
        assert_eq!(result, vec![EvaluationResult::Number(5.0), EvaluationResult::Number(8.0)], "Let and variable should be handled correctly");
    }

    #[test]
    fn test_evaluate_with_let_and_variable_and_undefined_variable() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let x = 5; x + y", &mut variables);
        // result should be exactly [EvaluationResult::Number(5.0), EvaluationResult::Error(_)]
        assert_eq!(result.len(), 2, "Result should contain exactly two elements");
        assert_eq!(result[0], EvaluationResult::Number(5.0));
        assert!(matches!(result[1], EvaluationResult::Error(_)));
    }

    #[test]
    fn test_evaluate_with_let_and_already_defined_variable() {
        let mut variables = VarTable(vec![Variable { label: "x".to_string(), value: 5.0 }]);
        let result = evaluate("let x = 10; x + 3", &mut variables);
        assert_eq!(result.len(), 2, "Result should contain exactly two elements");
        assert!(
            matches!(result[0], EvaluationResult::Error(_)),
            "Result should be an error for expression 'let x = 10; x + 3'"
        );
        assert_eq!(result[1], EvaluationResult::Number(8.0), "Result should be 8 for expression 'let x = 10; x + 3' when x is already defined as 5");
    }

    #[test]
    fn test_evaluate_with_multiple_statements() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let x = 5; x + 3; x * 2", &mut variables);
        assert_eq!(
            result,
            vec![
                EvaluationResult::Number(5.0),
                EvaluationResult::Number(8.0),
                EvaluationResult::Number(10.0),
            ],
            "Multiple statements should be evaluated correctly"
        );
    }

    #[test]
    fn test_evaluate_with_multiple_statements_and_undefined_variable() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let x = 5; x + 3; x * y", &mut variables);

        assert_eq!(result.len(), 3, "Result should contain exactly three elements");
        assert_eq!(result[0], EvaluationResult::Number(5.0));
        assert_eq!(result[1], EvaluationResult::Number(8.0));
        assert!(matches!(result[2], EvaluationResult::Error(_)));
    }

    #[test]
    fn test_evaluate_assign_use_change_use() {
        let mut variables = VarTable(vec![]);
        let result = evaluate("let x = 5; x/-10; x = 10; x + 3", &mut variables);
        assert_eq!(
            result,
            vec![
                EvaluationResult::Number(5.0),
                EvaluationResult::Number(-0.5),
                EvaluationResult::Number(10.0),
                EvaluationResult::Number(13.0),
            ],
            "Assign, use, change, use should be evaluated correctly"
        );
    }
}
