use itertools::Itertools;
use nonempty::NonEmpty;
use pest::{error::Error, iterators::Pair, Parser};
use pest_derive::Parser;
use rust_decimal::Decimal;

use crate::data::Identifier;

#[derive(Parser)]
#[grammar = "merc.pest"]
pub(crate) struct MercParser;

pub(crate) fn parse(input: &str) -> Result<Parsed, Box<Error<Rule>>> {
    let file = MercParser::parse(Rule::file, input)?.next().unwrap();
    let statements = file
        .into_inner()
        .filter_map(|pair| match pair.as_rule() {
            Rule::entry => {
                let mut inner_rules = pair.into_inner();
                let comment = inner_rules.next().unwrap().as_str().trim();
                let comment = if !comment.is_empty() {
                    Some(
                        comment
                            .lines()
                            .filter(|line| !line.trim().is_empty())
                            .join("\n"),
                    )
                } else {
                    None
                };
                let accesses = {
                    let mut inner = inner_rules.next().unwrap().into_inner();

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

    pub(crate) fn into_string(self) -> Result<String, crate::Error> {
        let merc_value = crate::data::evaluate(self)
            .map_err(|error| crate::Error::EvaluationError(Box::new(error)))?;
        Ok(merc_value.print())
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
        Rule::string => ValueKind::String(parse_string(pair)),

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
    String(StringKind),
    Integer(isize),
    Decimal(Decimal),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub(crate) enum StringKind {
    SinglelineRaw(String),
    MultilineAbleRaw(String),
    SinglineEscaped(String),
    MultilineAbleEscaped(String),
}
impl StringKind {
    pub(crate) fn to_string(&self) -> String {
        match self {
            StringKind::SinglelineRaw(string)
            | StringKind::SinglineEscaped(string)
            | StringKind::MultilineAbleRaw(string)
            | StringKind::MultilineAbleEscaped(string) => string.clone(),
        }
    }
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
    ArrayAccessImplicit,
    ArrayAccessExplicit { key: Identifier },
}
fn parse_access(pair: Pair<Rule>) -> Access {
    let span: Span = pair.as_span().into();
    let kind = match pair.as_rule() {
        Rule::array_access_implicit => AccessKind::ArrayAccessImplicit,
        Rule::array_access_explicit => AccessKind::ArrayAccessExplicit {
            key: parse_identifier(pair.into_inner().next().unwrap()),
        },
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
        Rule::string => Identifier::Quoted(parse_string(pair)),
        rule => unreachable!("rule = {:?}", rule),
    }
}

fn parse_string(pair: Pair<Rule>) -> StringKind {
    let inner = pair.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::singleline_escaped_string => {
            StringKind::SinglineEscaped(trim_by_count(1, inner.as_str()))
        }
        Rule::singleline_raw_string => StringKind::SinglelineRaw(trim_by_count(1, inner.as_str())),
        Rule::multiline_able_raw_string => {
            StringKind::MultilineAbleRaw(trim_by_count(3, inner.as_str()))
        }
        Rule::multiline_able_escaped_string => {
            StringKind::MultilineAbleEscaped(trim_by_count(3, inner.as_str()))
        }
        rule => unreachable!("rule = {:?}", rule),
    }
}

/// Trim the first `count` characters and last `count` characters from the given string
pub(crate) fn trim_by_count(count: usize, s: &str) -> String {
    let char_count = s.chars().count();
    s.chars()
        .skip(count)
        .take(char_count.saturating_sub(count * 2))
        .collect::<String>()
}
