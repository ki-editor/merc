use crate::data::{EvaluateError, Identifier};
use itertools::Itertools;
use nonempty::NonEmpty;
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "merc.pest"]
pub(crate) struct MercParser;

pub(crate) fn parse(input: &str) -> Result<Parsed, crate::Error> {
    let file = MercParser::parse(Rule::file, input)
        .map_err(|error| crate::Error::ParseError(Box::new(error)))?
        .next()
        .unwrap();
    let statements = file
        .into_inner()
        .map(|pair| -> Result<_, EvaluateError> {
            match pair.as_rule() {
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
                        let access_head = parse_access(next)?;

                        let access_tail = inner.map(|pair| parse_access(pair)).try_collect()?;

                        NonEmpty {
                            head: access_head,
                            tail: access_tail,
                        }
                    };
                    let value = parse_value(inner_rules.next().unwrap())?;
                    Ok(Some(Statement::Entry(Entry {
                        comment,
                        accesses,
                        value,
                    })))
                }
                Rule::comment => Ok(Some(Statement::Comment(Comment(pair.as_str().to_string())))),
                Rule::EOI => Ok(None),
                _ => unreachable!(),
            }
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|evaluate_error| crate::Error::EvaluationError(Box::new(evaluate_error)))?
        .into_iter()
        .flatten()
        .collect();
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

fn parse_value(pair: Pair<Rule>) -> Result<EntryValue, EvaluateError> {
    let span = pair.as_span().into();
    let kind = match pair.as_rule() {
        Rule::string => ValueKind::String(parse_string(pair)?),

        Rule::number => {
            ValueKind::Decimal(str::parse::<serde_json::Number>(pair.as_str()).unwrap())
        }
        Rule::integer => ValueKind::Integer(str::parse::<isize>(pair.as_str()).unwrap()),
        Rule::boolean => ValueKind::Boolean(str::parse::<bool>(pair.as_str()).unwrap()),
        Rule::null => ValueKind::Null,
        rule => unreachable!("rule = {rule:?}"),
    };
    Ok(EntryValue { span, kind })
}

#[derive(Debug)]
pub(crate) enum ValueKind {
    String(MercString),
    Integer(isize),
    Decimal(serde_json::Number),
    Boolean(bool),
    Null,
}

#[derive(Debug, Clone)]
pub(crate) struct MercString {
    value: String,
}
impl MercString {
    pub(crate) fn new(kind: StringKind, span: Span, string: &str) -> Result<Self, EvaluateError> {
        let escape = |s: &str| -> Result<_, _> {
            unescaper::unescape(s).map_err(|error| EvaluateError::StringUnescapeError {
                span: span.clone(),
                error: error.to_string(),
            })
        };
        let check_multiline_format = |s: &str| {
            if s.contains('\n') {
                if !s.starts_with('\n') {
                    Err(EvaluateError::MultilineStringNotStartingWithNewline { span: span.clone() })
                } else if !s.ends_with('\n') {
                    Err(EvaluateError::MultilineStringNotEndingWithNewline { span: span.clone() })
                } else {
                    Ok(trim_by_count(1, s))
                }
            } else {
                Ok(s.to_string())
            }
        };
        let value = match kind {
            StringKind::SinglelineRaw => string.to_string(),
            StringKind::SinglineEscaped => escape(string)?,
            StringKind::MultilineAbleRaw => check_multiline_format(string)?,
            StringKind::MultilineAbleEscaped => check_multiline_format(&escape(string)?)?,
        };
        Ok(Self { value })
    }

    pub(crate) fn string_value(&self) -> String {
        self.value.clone()
    }

    pub(crate) fn display(&self) -> String {
        let s = self.string_value();
        fn serialize_string(s: &str) -> String {
            trim_by_count(
                1,
                &serde_json::to_string(&serde_json::Value::String(s.to_string())).unwrap(),
            )
        }
        if !s.contains('\n') && !s.contains('\'') {
            format!("\'{s}\'")
        } else if !s.contains("'''") && !s.contains('\n') {
            format!("\'''{s}\'''")
        } else if !s.contains("'''") {
            format!("'''\n{}\n'''", s)
        } else if s.contains('\n') {
            format!("\"\"\"\n{}\n\"\"\"", s)
        } else {
            let s = serialize_string(&s);
            format!("\"{}\"", s)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum StringKind {
    SinglelineRaw,
    MultilineAbleRaw,
    SinglineEscaped,
    MultilineAbleEscaped,
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
fn parse_access(pair: Pair<Rule>) -> Result<Access, EvaluateError> {
    let span: Span = pair.as_span().into();
    let kind = match pair.as_rule() {
        Rule::array_access_implicit => AccessKind::ArrayAccessImplicit,
        Rule::array_access_explicit => AccessKind::ArrayAccessExplicit {
            key: parse_identifier(pair.into_inner().next().unwrap())?,
        },
        Rule::object_access => AccessKind::ObjectAccess {
            key: parse_identifier(pair.into_inner().next().unwrap())?,
        },
        Rule::map_access => AccessKind::MapAccess {
            key: parse_identifier(pair.into_inner().next().unwrap())?,
        },
        rule => unreachable!("rule = {:?}", rule),
    };
    Ok(Access { kind, span })
}

fn parse_identifier(pair: Pair<Rule>) -> Result<Identifier, EvaluateError> {
    match pair.as_rule() {
        Rule::unquoted_identifier => Ok(Identifier::Unquoted(pair.as_str().to_string())),
        Rule::string => Ok(Identifier::Quoted(parse_string(pair)?)),
        rule => unreachable!("rule = {:?}", rule),
    }
}

fn parse_string(pair: Pair<Rule>) -> Result<MercString, EvaluateError> {
    let inner = pair.into_inner().next().unwrap();
    let string = inner.as_str();
    let (kind, string) = match inner.as_rule() {
        Rule::singleline_escaped_string => (StringKind::SinglineEscaped, trim_by_count(1, string)),
        Rule::singleline_raw_string => (StringKind::SinglelineRaw, trim_by_count(1, string)),
        Rule::multiline_able_raw_string => (StringKind::MultilineAbleRaw, trim_by_count(3, string)),
        Rule::multiline_able_escaped_string => {
            (StringKind::MultilineAbleEscaped, trim_by_count(3, string))
        }
        rule => unreachable!("rule = {:?}", rule),
    };
    MercString::new(kind, inner.as_span().into(), &string)
}

/// Trim the first `count` characters and last `count` characters from the given string
pub(crate) fn trim_by_count(count: usize, s: &str) -> String {
    let char_count = s.chars().count();
    s.chars()
        .skip(count)
        .take(char_count.saturating_sub(count * 2))
        .collect::<String>()
}
