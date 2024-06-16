use data::evaluate;
use parser::Rule;

mod data;
mod parser;

#[cfg(test)]
mod test_cases;

fn main() {
    marc_to_json(".x = 123").unwrap();
}

#[derive(Debug)]
enum Error {
    ParseError(Box<pest::error::Error<Rule>>),
    EvaluationError(Box<data::EvaluateError>),
}

fn marc_to_json(marc: &str) -> Result<serde_json::Value, String> {
    fn marc_to_json(marc: &str) -> Result<serde_json::Value, Error> {
        let parsed = parser::parse(marc).map_err(Error::ParseError)?;
        let marc_value =
            evaluate(parsed).map_err(|error| Error::EvaluationError(Box::new(error)))?;
        Ok(marc_value.into_json())
    }
    marc_to_json(marc).map_err(|err| err.display(marc))
}

impl Error {
    fn display(&self, source: &str) -> String {
        match self {
            Error::ParseError(error) => error.to_string(),
            Error::EvaluationError(error) => error.display(source),
        }
    }
}
