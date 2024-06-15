use data::{evaluate, EvaluationError};
use parser::Rule;

mod data;
mod parser;
mod parser_nom;

#[cfg(test)]
mod test_cases;

fn main() {
    println!("Hello, world!");
}

#[derive(Debug)]
enum Error {
    ParseError(pest::error::Error<Rule>),
    EvaluationError(data::EvaluationError),
}

fn marc_to_json(marc: &str) -> Result<serde_json::Value, Error> {
    let parsed = parser::parse(marc).map_err(Error::ParseError)?;
    let marc_value = evaluate(parsed).map_err(Error::EvaluationError)?;
    Ok(marc_value.into_json())
}
