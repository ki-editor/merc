use itertools::Itertools;
use nonempty::NonEmpty;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use rust_decimal::Decimal;

#[derive(Parser)]
#[grammar = "marc.pest"]
pub(crate) struct MarcParser;

pub(crate) fn parse(input: &str) -> anyhow::Result<Parsed> {
    let file = MarcParser::parse(Rule::file, input)?
        .into_iter()
        .next()
        .unwrap();
    let statements = file
        .into_inner()
        .into_iter()
        .filter_map(|pair| match pair.as_rule() {
            Rule::entry => {
                let mut inner_rules = pair.into_inner();
                let accesses = {
                    let mut inner = inner_rules.next().unwrap().into_inner();
                    let access_head = parse_access(inner.next().unwrap());

                    let access_tail = inner
                        .into_iter()
                        .map(|pair| parse_access(pair))
                        .collect_vec();

                    NonEmpty {
                        head: access_head,
                        tail: access_tail,
                    }
                };
                println!("\n\naccesses = {:?}", accesses);
                let value = parse_value(inner_rules.next().unwrap());
                Some(Statement::Entry(Entry { accesses, value }))
            }
            Rule::COMMENT => Some(Statement::Comment(Comment(pair.as_str().to_string()))),
            Rule::EOI => None,
            _ => unreachable!(),
        })
        .collect_vec();
    Ok(Parsed(statements))
}

struct Parsed(Vec<Statement>);

enum Statement {
    Entry(Entry),
    Comment(Comment),
}

struct Comment(String);

struct Entry {
    accesses: NonEmpty<Access>,
    value: Value,
}

fn parse_value(pair: Pair<Rule>) -> Value {
    println!("parse_value pair = {pair:?}");
    match pair.as_rule() {
        Rule::string_inner => Value::String(pair.as_str().to_string()),
        Rule::multiline_string_inner => Value::MultilineString(pair.as_str().to_string()),
        Rule::number => Value::Decimal(str::parse::<Decimal>(pair.as_str()).unwrap()),
        Rule::integer => Value::Integer(str::parse::<isize>(pair.as_str()).unwrap()),
        Rule::boolean => Value::Boolean(str::parse::<bool>(pair.as_str()).unwrap()),
        Rule::null => Value::Null,
        rule => unreachable!("rule = {rule:?}"),
    }
}

#[derive(Debug)]
enum Value {
    String(String),
    MultilineString(String),
    Integer(isize),
    Decimal(Decimal),
    Boolean(bool),
    Null,
}

#[derive(Debug)]
enum Access {
    ObjectAccess { key: String },
    MapAccess { key: String },
    ArrayAccessNext,
    ArrayAccessCurrent,
}

fn parse_access(pair: Pair<Rule>) -> Access {
    match pair.as_rule() {
        Rule::array_access_next => Access::ArrayAccessNext,
        Rule::array_access_current => Access::ArrayAccessCurrent,
        Rule::object_access => Access::ObjectAccess {
            key: pair.into_inner().next().unwrap().as_str().to_string(),
        },
        Rule::map_access => Access::MapAccess {
            key: pair.into_inner().next().unwrap().as_str().to_string(),
        },
        rule => unreachable!("rule = {:?}", rule),
    }
}

#[cfg(test)]
mod test_parser {
    use super::*;
    use pest::Parser;
    use pest_derive::Parser;

    #[test]
    fn map() {
        let input = r#"
# Map
.materials{metal}.reflectivity = 1.0
.materials{metal}.metallic = true
.materials{plastic}.reflectivity = 0.5
.materials{plastic}.conductivity = null

# Array of objects
.entities[i].name = "hero"
.entities[ ].material = "metal"

.entities[i].name = "monster"
.entities[ ].material = "plastic"

# Multiline string
.description = """
These are common materials.
They are found on Earth.
"""

"#
        .trim();
        parse(&input).unwrap();
    }
}
