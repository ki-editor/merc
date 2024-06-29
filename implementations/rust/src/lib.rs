use data::evaluate;
use parser::Rule;
use wasm_bindgen::prelude::*;

mod data;
mod parser;

#[cfg(test)]
mod test_cases;

#[derive(Debug)]
pub(crate) enum Error {
    ParseError(Box<pest::error::Error<Rule>>),
    EvaluationError(Box<data::EvaluateError>),
}

#[wasm_bindgen]
pub fn merc_to_json_string(merc: &str) -> Result<String, String> {
    merc_to_json(merc)
        .map_err(|err| err.display(merc))
        .and_then(|json| serde_json::to_string_pretty(&json).map_err(|err| err.to_string()))
}

fn merc_to_json(merc: &str) -> Result<serde_json::Value, Error> {
    let parsed = parser::parse(merc)?;
    let merc_value = evaluate(parsed).map_err(|error| Error::EvaluationError(Box::new(error)))?;
    Ok(merc_value.into_json())
}

#[wasm_bindgen]
pub fn json_to_merc_string(json: &str) -> Result<String, String> {
    json_to_merc(json)
        .map(|merc| merc.print())
        .map_err(|err| err.to_string())
}

fn json_to_merc(json: &str) -> anyhow::Result<data::Value> {
    let parsed = serde_json::from_str(json)?;
    data::Value::from_json(parsed).map_err(|err| anyhow::anyhow!("{}", err.display(json)))
}

#[wasm_bindgen]
pub fn json_to_yaml_string(json: &str) -> Result<String, String> {
    (|| -> anyhow::Result<String> {
        {
            let parsed = serde_json::from_str(json)?;
            Ok(serde_yaml::to_string(&serde_yaml::to_value::<
                serde_yaml::Value,
            >(parsed)?)?)
        }
    })()
    .map_err(|err| err.to_string())
}

#[wasm_bindgen]
pub fn json_to_toml_string(json: &str) -> Result<String, String> {
    (|| -> anyhow::Result<String> {
        {
            let parsed: serde_json::Value = serde_json::from_str(json)?;
            Ok(toml::to_string_pretty(&parsed)?)
        }
    })()
    .map_err(|err| err.to_string())
}

#[wasm_bindgen]
pub fn toml_to_json_string(toml: &str) -> Result<String, String> {
    (|| -> anyhow::Result<String> {
        {
            let parsed: serde_json::Value = toml::from_str(toml)?;
            Ok(serde_json::to_string_pretty(&parsed)?)
        }
    })()
    .map_err(|err| err.to_string())
}

#[wasm_bindgen]
pub fn yaml_to_json_string(yaml: &str) -> Result<String, String> {
    (|| -> anyhow::Result<String> {
        {
            let parsed = serde_yaml::from_str(yaml)?;
            Ok(serde_json::to_string_pretty(&serde_json::to_value::<
                serde_json::Value,
            >(parsed)?)?)
        }
    })()
    .map_err(|err| err.to_string())
}

impl Error {
    fn display(&self, source: &str) -> String {
        match self {
            Error::ParseError(error) => error.to_string(),
            Error::EvaluationError(error) => error.display(source),
        }
    }
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[cfg(test)]
mod test_lib {
    use super::*;
    #[test]
    fn test_json_to_toml_string_1() {
        assert_eq!(
            json_to_toml_string(r#"{"x": {"y": 2}}"#).unwrap(),
            "[x]\ny = 2\n"
        )
    }
    #[test]
    fn test_toml_to_json_string_1() {
        assert_eq!(
            toml_to_json_string("[x]\ny = 2\n").unwrap(),
            serde_json::to_string_pretty(&serde_json::json!({"x": {"y": 2}})).unwrap()
        )
    }
}

#[wasm_bindgen]
pub fn format_merc(merc: &str) -> Result<String, String> {
    fn format_merc(merc: &str) -> Result<String, Error> {
        let parsed = parser::parse(merc)?;
        parsed.into_string()
    }
    format_merc(merc).map_err(|err| err.display(merc))
}
