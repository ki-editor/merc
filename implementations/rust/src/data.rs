use annotate_snippets::{Annotation, Level};
use indexmap::IndexMap;
use itertools::Itertools;
use rust_decimal::Decimal;

use crate::parser::{Access, AccessKind, Parsed, Span};

#[derive(Clone, Debug)]
pub(crate) struct Value {
    kind: ValueKind,
    inferred_at: Span,
}

#[derive(Clone, Debug)]
enum ValueKind {
    Scalar {
        /// Comment is attached to scalar value
        /// because each scalar value represents an entry
        comment: Option<String>,
        kind: ValueScalarKind,
    },
    ArrayLike(ArrayLike),
    MapLike(MapLike),
    Uninitialized,
}

#[derive(Clone, Debug)]
enum ValueScalarKind {
    String(String),
    Integer(isize),
    Decimal(Decimal),
    Null,
    Boolean(bool),
}

impl ValueKind {
    fn typ(&self) -> Type {
        match self {
            ValueKind::ArrayLike(ArrayLike {
                kind: ArrayKind::Array,
                ..
            }) => Type::Array,
            ValueKind::ArrayLike(ArrayLike {
                kind: ArrayKind::Tuple,
                ..
            }) => Type::Tuple,
            ValueKind::MapLike(MapLike {
                kind: MapKind::Object,
                ..
            }) => Type::Object,
            ValueKind::MapLike(MapLike {
                kind: MapKind::Map, ..
            }) => Type::Map,
            ValueKind::Scalar { kind, .. } => match kind {
                ValueScalarKind::String(_) => Type::String,
                ValueScalarKind::Integer(_) => Type::Integer,
                ValueScalarKind::Decimal(_) => Type::Decimal,
                ValueScalarKind::Null => Type::Null,
                ValueScalarKind::Boolean(_) => Type::Boolean,
            },
            ValueKind::Uninitialized => unreachable!(),
        }
    }

    fn is_scalar(&self) -> bool {
        matches!(self, ValueKind::Scalar { .. })
    }

    fn into_json(self) -> serde_json::Value {
        match self {
            ValueKind::ArrayLike(array_like) => array_like.into_json(),
            ValueKind::MapLike(map_like) => map_like.into_json(),
            ValueKind::Scalar { kind, .. } => match kind {
                ValueScalarKind::String(string) => serde_json::Value::String(string),
                ValueScalarKind::Integer(integer) => serde_json::Value::Number(integer.into()),
                ValueScalarKind::Decimal(decimal) => str::parse(&decimal.to_string())
                    .map(serde_json::Value::Number)
                    .unwrap_or_else(|_| serde_json::Value::String(decimal.to_string())),
                ValueScalarKind::Null => serde_json::Value::Null,
                ValueScalarKind::Boolean(boolean) => serde_json::Value::Bool(boolean),
            },
            ValueKind::Uninitialized => unreachable!(),
        }
    }

    fn to_string_entries(&self, parent_path: &str) -> Vec<StringEntry> {
        match self {
            ValueKind::ArrayLike(array) => array
                .array
                .iter()
                .flat_map(|value| {
                    value
                        .kind
                        .to_string_entries("")
                        .into_iter()
                        .enumerate()
                        .map(|(index, StringEntry { comment, entry })| {
                            let path = match (&array.kind, index == 0) {
                                (ArrayKind::Array, true) => "[i]",
                                (ArrayKind::Array, false) => "[ ]",
                                (ArrayKind::Tuple, true) => "(i)",
                                (ArrayKind::Tuple, false) => "( )",
                            };
                            let entry = format!("{parent_path}{path}{entry}");
                            StringEntry { comment, entry }
                        })
                })
                .collect(),
            ValueKind::MapLike(map) => map
                .map
                .iter()
                .sorted_by_key(|(key, _)| key.string_value())
                .flat_map(|(key, value)| {
                    let path = match map.kind {
                        MapKind::Object => format!(".{}", key.display()),
                        MapKind::Map => format!("{{{}}}", key.display()),
                    };
                    value
                        .kind
                        .to_string_entries(&format!("{parent_path}{path}"))
                })
                .collect(),
            ValueKind::Scalar { comment, kind } => {
                let entry = match kind {
                    ValueScalarKind::String(s) => {
                        fn serialize_string(s: &str) -> String {
                            serde_json::to_string(&serde_json::Value::String(s.to_string()))
                                .unwrap()
                                .trim_matches('"')
                                .to_string()
                        }
                        let s = if s.contains('\n') {
                            let s = s.lines().map(serialize_string).join("\n");
                            format!("\"\"\"\n{}\n\"\"\"", s)
                        } else {
                            let s = serialize_string(s);
                            format!("\"{}\"", s)
                        };
                        format!("{parent_path} = {}", s)
                    }
                    ValueScalarKind::Integer(i) => format!("{parent_path} = {:?}", i),
                    ValueScalarKind::Decimal(d) => format!("{parent_path} = {:?}", d),
                    ValueScalarKind::Null => {
                        format!("{parent_path} = null")
                    }
                    ValueScalarKind::Boolean(b) => format!("{parent_path} = {:?}", b),
                };
                Some(StringEntry {
                    comment: comment.clone(),
                    entry,
                })
                .into_iter()
                .collect_vec()
            }
            ValueKind::Uninitialized => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
struct ArrayLike {
    kind: ArrayKind,
    array: Vec<Value>,
}
impl ArrayLike {
    fn new(kind: ArrayKind) -> Self {
        Self {
            kind,
            array: Default::default(),
        }
    }

    fn push_new(mut self, tail: &[Access], value: Value) -> Result<ArrayLike, EvaluateError> {
        let new_element = Value::uninitialized().set(tail, value)?;
        self.array.push(new_element);
        Ok(self)
    }

    fn set_last(mut self, tail: &[Access], value: Value) -> Result<ArrayLike, EvaluateError> {
        if let Some(last) = self.array.pop() {
            self.array.push(last.set(tail, value)?);
            Ok(self)
        } else {
            unreachable!()
        }
    }

    fn into_json(self) -> serde_json::Value {
        serde_json::Value::Array(
            self.array
                .into_iter()
                .map(|value| value.into_json())
                .collect_vec(),
        )
    }
}
#[derive(Debug, Clone)]
enum ArrayKind {
    Array,
    Tuple,
}
#[derive(Debug, Clone)]
struct MapLike {
    kind: MapKind,
    map: IndexMap<Identifier, Value>,
}
#[derive(Debug, Clone)]
pub(crate) enum Identifier {
    Quoted(String),
    Unquoted(String),
}
impl std::hash::Hash for Identifier {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.string_value().hash(state)
    }
}
impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.string_value().eq(&other.string_value())
    }
}
impl Eq for Identifier {}
impl Identifier {
    fn string_value(&self) -> String {
        match self {
            Identifier::Quoted(string) | Identifier::Unquoted(string) => string.to_string(),
        }
    }
    fn display(&self) -> String {
        let string = self.string_value();
        if Self::needs_quote(&string) {
            format!("\"{}\"", string)
        } else {
            string.to_string()
        }
    }

    fn from_str(key: &str) -> Identifier {
        if Self::needs_quote(key) {
            Identifier::Quoted(key.to_string())
        } else {
            Identifier::Unquoted(key.to_string())
        }
    }

    fn needs_quote(key: &str) -> bool {
        !key.chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }
}
impl MapLike {
    fn new(kind: MapKind) -> Self {
        Self {
            kind,
            map: Default::default(),
        }
    }

    fn set(self, key: Identifier, tail: &[Access], value: Value) -> Result<Self, EvaluateError> {
        let mut map = self.map;
        let map = if let Some(current_value) = map.shift_remove(&key) {
            if current_value.is_scalar() && value.is_scalar() {
                return Err(EvaluateError::DuplicateAssignment {
                    previously_assigned_at: current_value.inferred_at,
                    now_assigned_again_at: value.inferred_at,
                });
            }
            map.insert(key, current_value.set(tail, value)?);
            map
        } else {
            map.insert(key, Value::uninitialized().set(tail, value)?);
            map
        };
        Ok(Self {
            kind: self.kind,
            map,
        })
    }

    fn into_json(self) -> serde_json::Value {
        serde_json::Value::Object(
            self.map
                .into_iter()
                .map(|(key, value)| (key.string_value(), value.into_json()))
                .collect(),
        )
    }
}

#[derive(Debug, Clone)]
enum MapKind {
    Object,
    Map,
}
impl Value {
    pub(crate) fn print(&self) -> String {
        self.kind
            .to_string_entries("")
            .into_iter()
            .map(|StringEntry { comment, entry }| {
                let comment = comment
                    .as_ref()
                    .map(|comment| format!("\n{}\n", comment))
                    .unwrap_or_default();
                format!("{comment}{entry}")
            })
            .join("\n")
            .trim()
            .to_string()
    }
    pub(crate) fn from_json(json: serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value {
                inferred_at: Span::default(),
                kind: ValueKind::Scalar {
                    comment: None,
                    kind: ValueScalarKind::Null,
                },
            },
            serde_json::Value::Bool(boolean) => Value {
                inferred_at: Span::default(),
                kind: ValueKind::Scalar {
                    comment: None,
                    kind: ValueScalarKind::Boolean(boolean),
                },
            },
            serde_json::Value::Number(number) => Value {
                inferred_at: Span::default(),
                kind: ValueKind::Scalar {
                    comment: None,
                    kind: ValueScalarKind::Decimal(
                        Decimal::from_str_exact(&number.to_string()).unwrap(),
                    ),
                },
            },
            serde_json::Value::String(string) => Value {
                inferred_at: Span::default(),
                kind: ValueKind::Scalar {
                    comment: None,
                    kind: ValueScalarKind::String(string.to_string()),
                },
            },
            serde_json::Value::Array(array) => Value {
                inferred_at: Span::default(),
                kind: ValueKind::ArrayLike(ArrayLike {
                    kind: ArrayKind::Array,
                    array: array.into_iter().map(Value::from_json).collect_vec(),
                }),
            },
            serde_json::Value::Object(map) => {
                Value {
                    inferred_at: Span::default(),
                    kind: ValueKind::MapLike(MapLike {
                        // We default to Object instead of Map, because I think Object is more common than Map
                        kind: MapKind::Object,
                        map: IndexMap::from_iter(map.into_iter().map(|(key, value)| {
                            (Identifier::from_str(&key), Value::from_json(value))
                        })),
                    }),
                }
            }
        }
    }
    fn update(self, entry: crate::parser::Entry) -> Result<Value, EvaluateError> {
        self.set(
            &entry.accesses.into_iter().collect_vec(),
            evaluate_value(entry.comment, entry.value)?,
        )
    }

    fn set(self, accesses: &[Access], value: Value) -> Result<Value, EvaluateError> {
        let Some((head, tail)) = accesses.split_first() else {
            return Ok(value);
        };

        let span = head.span.clone();

        match (&self.kind, &head.kind) {
            (ValueKind::Uninitialized, AccessKind::MapAccess { .. }) => Value {
                inferred_at: span,
                kind: ValueKind::MapLike(MapLike::new(MapKind::Map)),
            }
            .set(accesses, value),
            (ValueKind::Uninitialized, AccessKind::ObjectAccess { .. }) => Value {
                inferred_at: span,
                kind: ValueKind::MapLike(MapLike::new(MapKind::Object)),
            }
            .set(accesses, value),
            (ValueKind::Uninitialized, AccessKind::ArrayAccessNew) => Value {
                inferred_at: span,
                kind: ValueKind::ArrayLike(ArrayLike::new(ArrayKind::Array)),
            }
            .set(accesses, value),
            (ValueKind::Uninitialized, AccessKind::TupleAccessNew) => Value {
                inferred_at: span,
                kind: ValueKind::ArrayLike(ArrayLike::new(ArrayKind::Tuple)),
            }
            .set(accesses, value),
            (ValueKind::Uninitialized, AccessKind::ArrayAccessLast) => {
                Err(EvaluateError::LastArrayElementNotFound { span })
            }
            (ValueKind::ArrayLike(array), AccessKind::ArrayAccessNew) => Ok(self
                .clone()
                .update_value(ValueKind::ArrayLike(array.clone().push_new(tail, value)?))),
            (ValueKind::ArrayLike(array), AccessKind::ArrayAccessLast) => Ok(self
                .clone()
                .update_value(ValueKind::ArrayLike(array.clone().set_last(tail, value)?))),
            (ValueKind::ArrayLike(array), AccessKind::TupleAccessNew) => Ok(self
                .clone()
                .update_value(ValueKind::ArrayLike(array.clone().push_new(tail, value)?))),
            (ValueKind::ArrayLike(array), AccessKind::TupleAccessLast) => Ok(self
                .clone()
                .update_value(ValueKind::ArrayLike(array.clone().set_last(tail, value)?))),
            (
                ValueKind::MapLike(
                    object @ MapLike {
                        kind: MapKind::Object,
                        ..
                    },
                ),
                AccessKind::ObjectAccess { key },
            ) => Ok(self
                .clone()
                .update_value(ValueKind::MapLike(object.clone().set(
                    key.clone(),
                    tail,
                    value,
                )?))),
            (
                ValueKind::MapLike(
                    object @ MapLike {
                        kind: MapKind::Map, ..
                    },
                ),
                AccessKind::MapAccess { key },
            ) => Ok(self
                .clone()
                .update_value(ValueKind::MapLike(object.clone().set(
                    key.clone(),
                    tail,
                    value,
                )?))),
            (expected_value, actual_access) => {
                Err(EvaluateError::TypeMismatch(Box::new(TypeMismatch::new(
                    expected_value.typ(),
                    self.inferred_at,
                    actual_access.typ(),
                    head.span.clone(),
                ))))
            }
        }
    }

    pub(crate) fn into_json(self) -> serde_json::Value {
        self.kind.into_json()
    }

    fn uninitialized() -> Value {
        Value {
            kind: ValueKind::Uninitialized,
            inferred_at: Span::default(),
        }
    }

    fn update_value(self, kind: ValueKind) -> Value {
        Value { kind, ..self }
    }

    fn is_scalar(&self) -> bool {
        self.kind.is_scalar()
    }
}

impl AccessKind {
    pub(crate) fn typ(&self) -> crate::data::Type {
        match self {
            AccessKind::ObjectAccess { .. } => Type::Object,
            AccessKind::MapAccess { .. } => Type::Map,
            AccessKind::ArrayAccessNew | AccessKind::ArrayAccessLast => Type::Array,
            AccessKind::TupleAccessLast | AccessKind::TupleAccessNew => Type::Tuple,
        }
    }
}

impl EvaluateError {
    pub(crate) fn display(&self, source: &str) -> String {
        use annotate_snippets::{Renderer, Snippet};
        let annotations: Vec<Annotation> = self.annotations();
        let title = self.title();

        let message = Level::Error.title(title).snippet(
            annotations
                .into_iter()
                .fold(Snippet::source(source).fold(true), |result, annotation| {
                    result.annotation(annotation)
                }),
        );

        Renderer::plain().render(message).to_string()
    }

    fn annotations(&self) -> Vec<Annotation> {
        match self {
            EvaluateError::TypeMismatch(type_mismatch) => type_mismatch.annotations(),
            EvaluateError::LastArrayElementNotFound { span } => [
                Level::Error
                    .span(span.byte_range())
                    .label("Last array element not found."),
                Level::Help
                    .span(span.byte_range())
                    .label("Change `[ ]` to `[i]`"),
            ]
            .into_iter()
            .collect(),
            EvaluateError::DuplicateAssignment {
                previously_assigned_at,
                now_assigned_again_at,
            } => [
                Level::Info
                    .span(previously_assigned_at.byte_range())
                    .label("A value was previously assigned at this path."),
                Level::Error
                    .span(now_assigned_again_at.byte_range())
                    .label("Attempting to assign a new value at the same path is not allowed."),
            ]
            .into_iter()
            .collect_vec(),
            EvaluateError::StringUnescapeError { span, error } => {
                [Level::Error.span(span.byte_range()).label(error)]
                    .into_iter()
                    .collect_vec()
            }
        }
    }

    fn title(&self) -> &'static str {
        match self {
            EvaluateError::TypeMismatch(_) => "Type Mismatch",
            EvaluateError::LastArrayElementNotFound { .. } => "Last Array Element Not Found",
            EvaluateError::DuplicateAssignment { .. } => "Duplicate Assignment",
            EvaluateError::StringUnescapeError { .. } => "String Unsecape Error",
        }
    }
}

#[derive(Debug)]
pub(crate) enum EvaluateError {
    TypeMismatch(Box<TypeMismatch>),
    LastArrayElementNotFound {
        span: Span,
    },
    DuplicateAssignment {
        previously_assigned_at: Span,
        now_assigned_again_at: Span,
    },
    StringUnescapeError {
        span: Span,
        error: String,
    },
}
#[derive(Debug)]
pub(crate) struct TypeMismatch {
    expected_type_inferred_at: Span,
    actual_type_inferred_at: Span,
    info_label: String,
    error_label: String,
}
impl TypeMismatch {
    fn annotations(&self) -> Vec<Annotation> {
        [
            Level::Info
                .span(self.expected_type_inferred_at.byte_range())
                .label(&self.info_label),
            Level::Error
                .span(self.actual_type_inferred_at.byte_range())
                .label(&self.error_label),
        ]
        .into_iter()
        .collect_vec()
    }

    fn new(
        expected_type: Type,
        expected_type_inferred_at: Span,
        actual_type: Type,
        actual_type_inferred_at: Span,
    ) -> Self {
        Self {
            expected_type_inferred_at,
            actual_type_inferred_at,
            info_label: format!(
                "The type of the parent value was first inferred as {} due to this access.",
                expected_type.display()
            ),
            error_label: format!(
                "Error: this access treats the parent value as {}, but it was inferred as a different type.",
                actual_type.display()
            ),
        }
    }
}
#[derive(Debug)]
pub(crate) enum Type {
    Map,
    Array,
    Object,
    Tuple,
    String,
    Integer,
    Decimal,
    Null,
    Boolean,
}
impl Type {
    fn display(&self) -> &'static str {
        match self {
            Type::Map => "Map",
            Type::Array => "Array",
            Type::Object => "Object",
            Type::Tuple => "Tuple",
            Type::String => "String",
            Type::Integer => "Integer",
            Type::Decimal => "Decimal",
            Type::Null => "Null",
            Type::Boolean => "Boolean",
        }
    }
}

pub(crate) fn evaluate(parsed: Parsed) -> Result<Value, EvaluateError> {
    let result = Value::uninitialized();
    parsed
        .into_entries()
        .into_iter()
        .try_fold(result, |result, entry| result.update(entry))
}

fn evaluate_value(
    comment: Option<String>,
    value: crate::parser::EntryValue,
) -> Result<Value, EvaluateError> {
    let kind = match value.kind {
        crate::parser::ValueKind::MultilineString(string)
        | crate::parser::ValueKind::String(string) => {
            ValueScalarKind::String(unescaper::unescape(&string).map_err(|error| {
                EvaluateError::StringUnescapeError {
                    span: value.span.clone(),
                    error: error.to_string(),
                }
            })?)
        }
        crate::parser::ValueKind::Integer(integer) => ValueScalarKind::Integer(integer),
        crate::parser::ValueKind::Decimal(decimal) => ValueScalarKind::Decimal(decimal),
        crate::parser::ValueKind::Boolean(boolean) => ValueScalarKind::Boolean(boolean),
        crate::parser::ValueKind::Null => ValueScalarKind::Null,
    };
    Ok(Value {
        kind: ValueKind::Scalar { kind, comment },
        inferred_at: value.span,
    })
}

struct StringEntry {
    comment: Option<String>,
    entry: String,
}
