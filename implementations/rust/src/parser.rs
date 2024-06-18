use itertools::Itertools;
use nonempty::NonEmpty;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;
use rust_decimal::Decimal;

use crate::data::Identifier;

#[derive(Parser)]
#[grammar = "marc.pest"]
pub(crate) struct MarcParser;

pub(crate) fn parse(input: &str) -> Result<Parsed, Box<Error<Rule>>> {
    let file = MarcParser::parse(Rule::file, input)?.next().unwrap();
    let statements = file
        .into_inner()
        .filter_map(|pair| match pair.as_rule() {
            Rule::entry => {
                let mut inner_rules = pair.into_inner();
                let first_token = inner_rules.next().unwrap();
                let (comment, next_token) = if first_token.as_rule() == Rule::comment {
                    (
                        Some(first_token.as_str().to_string()),
                        inner_rules.next().unwrap(),
                    )
                } else {
                    (None, first_token)
                };
                let accesses = {
                    let mut inner = next_token.into_inner();

                    let next = inner.next().unwrap();
                    let access_head = parse_access(next);

                    let access_tail = inner.map(|pair| parse_access(pair)).collect_vec();

                    NonEmpty {
                        head: access_head,
                        tail: access_tail,
                    }
                };
                let value = parse_value(inner_rules.next().unwrap());
                Some(Statement::Entry(Entry {
                    comment,
                    accesses,
                    value,
                }))
            }
            Rule::comment => Some(Statement::Comment(Comment(pair.as_str().to_string()))),
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

    pub(crate) fn to_string(self) -> Result<String, crate::Error> {
        let marc_value = crate::data::evaluate(self)
            .map_err(|error| crate::Error::EvaluationError(Box::new(error)))?;
        Ok(marc_value.print())
    }
}

#[derive(Debug)]
enum Statement {
    Entry(Entry),
    Comment(Comment),
}

#[derive(Debug)]
struct Comment(String);

#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) comment: Option<String>,
    pub(crate) accesses: NonEmpty<Access>,
    pub(crate) value: EntryValue,
}

fn parse_value(pair: Pair<Rule>) -> EntryValue {
    let span = pair.as_span().into();
    let kind = match pair.as_rule() {
        Rule::string => ValueKind::String(pair.into_inner().next().unwrap().as_str().to_string()),
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
    start: usize,
    end: usize,
}
impl Span {
    pub(crate) fn byte_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    pub(crate) fn default() -> Span {
        Span { start: 0, end: 0 }
    }
}

impl From<pest::Span<'_>> for Span {
    fn from(value: pest::Span<'_>) -> Self {
        Self {
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
    ObjectAccess { key: Identifier },
    MapAccess { key: Identifier },
    ArrayAccessNew,
    ArrayAccessLast,
    TupleAccessLast,
    TupleAccessNew,
}
fn parse_access(pair: Pair<Rule>) -> Access {
    let span: Span = pair.as_span().into();
    let kind = match pair.as_rule() {
        Rule::array_access_new => AccessKind::ArrayAccessNew,
        Rule::array_access_last => AccessKind::ArrayAccessLast,
        Rule::tuple_access_new => AccessKind::TupleAccessNew,
        Rule::tuple_access_last => AccessKind::TupleAccessLast,
        Rule::object_access => AccessKind::ObjectAccess {
            key: parse_identifier(pair.into_inner().next().unwrap()),
        },
        Rule::map_access => AccessKind::MapAccess {
            key: parse_identifier(pair.into_inner().next().unwrap()),
        },
        rule => unreachable!("rule = {:?}", rule),
    };
    Access { kind, span }
}

fn parse_identifier(pair: Pair<Rule>) -> Identifier {
    match pair.as_rule() {
        Rule::unquoted_identifier => Identifier::Unquoted(pair.as_str().to_string()),
        Rule::string => Identifier::Quoted(pair.into_inner().next().unwrap().as_str().to_string()),
        rule => unreachable!("rule = {:?}", rule),
    }
}
