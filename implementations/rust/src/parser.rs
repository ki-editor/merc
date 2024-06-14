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

#[derive(Debug)]
pub(crate) struct Parsed(Vec<Statement>);
impl Parsed {
    pub(crate) fn into_entries(self) -> Vec<Entry> {
        self.0
            .into_iter()
            .filter_map(|statement| match statement {
                Statement::Entry(entry) => Some(entry),
                _ => None,
            })
            .collect_vec()
    }
}

#[derive(Debug)]
pub(crate) enum Statement {
    Entry(Entry),
    Comment(Comment),
}

#[derive(Debug)]
struct Comment(String);

#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) accesses: NonEmpty<Access>,
    pub(crate) value: EntryValue,
}

fn parse_value(pair: Pair<Rule>) -> EntryValue {
    let span = pair.as_span().clone().into();
    let kind = match pair.as_rule() {
        Rule::string_inner => ValueKind::String(pair.as_str().to_string()),
        Rule::multiline_string_inner => ValueKind::MultilineString(pair.as_str().to_string()),
        Rule::number => ValueKind::Decimal(str::parse::<Decimal>(pair.as_str()).unwrap()),
        Rule::integer => ValueKind::Integer(str::parse::<isize>(pair.as_str()).unwrap()),
        Rule::boolean => ValueKind::Boolean(str::parse::<bool>(pair.as_str()).unwrap()),
        Rule::null => ValueKind::Null,
        rule => unreachable!("rule = {rule:?}"),
    };
    EntryValue { span, kind }
}

#[derive(Debug)]
pub(crate) enum ValueKind {
    String(String),
    MultilineString(String),
    Integer(isize),
    Decimal(Decimal),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub(crate) struct Span {
    input: String,
    start: usize,
    end: usize,
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span<'_>) -> Self {
        Self {
            input: value.as_str().to_string(),
            start: value.start(),
            end: value.end(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct EntryValue {
    pub(crate) span: Span,
    pub(crate) kind: ValueKind,
}

#[derive(Debug, Clone)]
pub(crate) struct Access {
    pub(crate) span: Span,
    pub(crate) kind: AccessKind,
}
#[derive(Debug, Clone)]
pub(crate) enum AccessKind {
    ObjectAccess { key: String },
    MapAccess { key: String },
    ArrayAccessNew,
    ArrayAccessLast,
}
fn parse_access(pair: Pair<Rule>) -> Access {
    let span: Span = pair.as_span().into();
    println!("span = {span:?}");
    let kind = match pair.as_rule() {
        Rule::array_access_next => AccessKind::ArrayAccessNew,
        Rule::array_access_current => AccessKind::ArrayAccessLast,
        Rule::object_access => AccessKind::ObjectAccess {
            key: pair.into_inner().next().unwrap().as_str().to_string(),
        },
        Rule::map_access => AccessKind::MapAccess {
            key: pair.into_inner().next().unwrap().as_str().to_string(),
        },
        rule => unreachable!("rule = {:?}", rule),
    };
    Access { kind, span }
}

#[cfg(test)]
mod test_parser {
    use crate::data::evaluate;

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
        let parsed = parse(&input).unwrap();
        let value = evaluate(parsed).unwrap();
        println!("value = {value:?}");
    }
}
