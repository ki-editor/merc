use crate::parser::{Access, AccessKind, MercString, Parsed, Span, StringKind};
use annotate_snippets::{Annotation, Level};
use indexmap::IndexMap;
use itertools::Itertools;

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
    MapLike(MapLike),
    Uninitialized,
}

#[derive(Clone, Debug)]
enum ValueScalarKind {
    String(MercString),
    Integer(isize),
    Number(serde_json::Number),
    Null,
    Boolean(bool),
}

impl ValueKind {
    fn typ(&self) -> Type {
        match self {
            ValueKind::MapLike(MapLike {
                kind: MapKind::Object,
                ..
            }) => Type::Object,
            ValueKind::MapLike(MapLike {
                kind: MapKind::Map, ..
            }) => Type::Map,
            ValueKind::MapLike(MapLike {
                kind: MapKind::Array,
                ..
            }) => Type::Array,
            ValueKind::Scalar { kind, .. } => match kind {
                ValueScalarKind::String(_) => Type::String,
                ValueScalarKind::Integer(_) => Type::Integer,
                ValueScalarKind::Number(_) => Type::Decimal,
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
            ValueKind::MapLike(map_like) => map_like.into_json(),
            ValueKind::Scalar { kind, .. } => match kind {
                ValueScalarKind::String(string) => serde_json::Value::String(string.string_value()),
                ValueScalarKind::Integer(integer) => serde_json::Value::Number(integer.into()),
                ValueScalarKind::Number(decimal) => str::parse(&decimal.to_string())
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
            ValueKind::MapLike(map) => map
                .map
                .iter()
                .enumerate()
                .sorted_by_key(|(index, (key, _))| match map.kind {
                    MapKind::Object | MapKind::Map => key.string_value(),
                    MapKind::Array => index.to_string(),
                })
                .flat_map(|(_, (key, value))| {
                    let path = match map.kind {
                        MapKind::Object => format!(".{}", key.display()),
                        MapKind::Map => format!("{{{}}}", key.display()),
                        MapKind::Array => format!("[{}]", key.display()),
                    };
                    value
                        .kind
                        .to_string_entries(&format!("{parent_path}{path}"))
                })
                .collect(),
            ValueKind::Scalar { comment, kind } => {
                let entry = match kind {
                    ValueScalarKind::String(s) => format!("{parent_path} = {}", s.display()),
                    ValueScalarKind::Integer(i) => format!("{parent_path} = {:?}", i),
                    ValueScalarKind::Number(d) => {
                        format!("{parent_path} = {}", serde_json::to_string(d).unwrap())
                    }
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
struct MapLike {
    kind: MapKind,
    map: IndexMap<MapKey, Value>,
}

#[derive(Debug, Clone)]
pub(crate) enum MapKey {
    Implicit(MapKeyImplicit),
    Explicit(Identifier),
}

use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

fn increment_counter() -> usize {
    COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Copy, Hash, Default)]
pub(crate) struct MapKeyImplicit(usize);
impl MapKeyImplicit {
    pub(crate) fn new() -> MapKeyImplicit {
        MapKeyImplicit(increment_counter())
    }

    fn string_value(&self) -> String {
        self.0.to_string()
    }
}
impl MapKey {
    fn display(&self) -> String {
        match self {
            MapKey::Implicit(_) => "+".to_string(),
            MapKey::Explicit(identifier) => identifier.display(),
        }
    }

    fn string_value(&self) -> String {
        match self {
            MapKey::Implicit(index) => index.string_value(),
            MapKey::Explicit(identifier) => identifier.string_value(),
        }
    }
}
impl std::hash::Hash for MapKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            MapKey::Implicit(index) => index.hash(state),
            MapKey::Explicit(identifier) => identifier.hash(state),
        }
    }
}
impl PartialEq for MapKey {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (MapKey::Implicit(a), MapKey::Implicit(b)) => a == b,
            (MapKey::Explicit(a), MapKey::Explicit(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for MapKey {}

#[derive(Debug, Clone)]
pub(crate) enum Identifier {
    Quoted(MercString),
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
            Identifier::Quoted(string_kind) => string_kind.string_value(),
            Identifier::Unquoted(string) => string.to_string(),
        }
    }
    fn display(&self) -> String {
        match self {
            Identifier::Quoted(string) => {
                let s = string.string_value();
                if Self::needs_quote(&s) {
                    string.display()
                } else {
                    s
                }
            }
            Identifier::Unquoted(unquoted) => unquoted.to_string(),
        }
    }

    fn from_str(key: &str) -> Result<Identifier, EvaluateError> {
        Ok(if Self::needs_quote(key) {
            Identifier::Quoted(MercString::new(
                StringKind::SinglelineRaw,
                Span::default(),
                key,
            )?)
        } else {
            Identifier::Unquoted(key.to_string())
        })
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

    fn set(self, key: MapKey, tail: &[Access], value: Value) -> Result<Self, EvaluateError> {
        let mut map = self.map;
        let map = if let Some(current_value) = map.get_mut(&key) {
            if current_value.is_scalar() && value.is_scalar() {
                return Err(EvaluateError::DuplicateAssignment {
                    previously_assigned_at: current_value.inferred_at.clone(),
                    now_assigned_again_at: value.inferred_at,
                });
            }
            *current_value = current_value.clone().set(tail, value)?;
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
        match self.kind {
            MapKind::Object | MapKind::Map => serde_json::Value::Object(
                self.map
                    .into_iter()
                    .map(|(key, value)| (key.string_value(), value.into_json()))
                    .collect(),
            ),
            MapKind::Array => serde_json::Value::Array(
                self.map
                    .into_iter()
                    .map(|(_, value)| value.into_json())
                    .collect(),
            ),
        }
    }
}

#[derive(Debug, Clone)]
enum MapKind {
    Object,
    Map,
    Array,
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
    pub(crate) fn from_json(json: serde_json::Value) -> Result<Value, EvaluateError> {
        let result = match json {
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
                    kind: ValueScalarKind::Number(number),
                },
            },
            serde_json::Value::String(string) => Value {
                inferred_at: Span::default(),
                kind: ValueKind::Scalar {
                    comment: None,
                    kind: ValueScalarKind::String(MercString::new(
                        StringKind::SinglelineRaw,
                        Span::default(),
                        &string,
                    )?),
                },
            },
            serde_json::Value::Array(array) => {
                Value {
                    inferred_at: Span::default(),
                    kind: ValueKind::MapLike(MapLike {
                        // We default to Array instead of Set, because Set is too restrictive
                        kind: MapKind::Array,
                        map: IndexMap::from_iter(
                            array
                                .into_iter()
                                .enumerate()
                                .map(|(index, value)| {
                                    // If the `value` only needs one line of MERC to be represented
                                    // then use implicit keys, otherwise use explicit keys
                                    let key = match &value {
                                        serde_json::Value::Null
                                        | serde_json::Value::Bool(_)
                                        | serde_json::Value::Number(_)
                                        | serde_json::Value::String(_) => {
                                            MapKey::Implicit(MapKeyImplicit::new())
                                        }
                                        serde_json::Value::Array(values) => {
                                            if values.len() <= 1 {
                                                MapKey::Implicit(MapKeyImplicit::new())
                                            } else {
                                                MapKey::Explicit(Identifier::Unquoted(
                                                    index.to_string(),
                                                ))
                                            }
                                        }
                                        serde_json::Value::Object(map) => {
                                            if map.len() <= 1 {
                                                MapKey::Implicit(MapKeyImplicit::new())
                                            } else {
                                                MapKey::Explicit(Identifier::Unquoted(
                                                    index.to_string(),
                                                ))
                                            }
                                        }
                                    };
                                    Ok((key, Value::from_json(value)?))
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        ),
                    }),
                }
            }
            serde_json::Value::Object(map) => {
                Value {
                    inferred_at: Span::default(),
                    kind: ValueKind::MapLike(MapLike {
                        // We default to Object instead of Map, because I think Object is more common than Map
                        kind: MapKind::Object,
                        map: IndexMap::from_iter(
                            map.into_iter()
                                .map(|(key, value)| {
                                    Ok((
                                        MapKey::Explicit(Identifier::from_str(&key)?),
                                        Value::from_json(value)?,
                                    ))
                                })
                                .collect::<Result<Vec<_>, _>>()?,
                        ),
                    }),
                }
            }
        };
        Ok(result)
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
            (
                ValueKind::Uninitialized,
                AccessKind::ArrayAccessImplicit | AccessKind::ArrayAccessExplicit { .. },
            ) => Value {
                inferred_at: span,
                kind: ValueKind::MapLike(MapLike::new(MapKind::Array)),
            }
            .set(accesses, value),
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
                    MapKey::Explicit(key.clone()),
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
                    MapKey::Explicit(key.clone()),
                    tail,
                    value,
                )?))),
            (
                ValueKind::MapLike(
                    object @ MapLike {
                        kind: MapKind::Array,
                        ..
                    },
                ),
                AccessKind::ArrayAccessExplicit { key },
            ) => Ok(self
                .clone()
                .update_value(ValueKind::MapLike(object.clone().set(
                    MapKey::Explicit(key.clone()),
                    tail,
                    value,
                )?))),
            (
                ValueKind::MapLike(
                    object @ MapLike {
                        kind: MapKind::Array,
                        ..
                    },
                ),
                AccessKind::ArrayAccessImplicit,
            ) => Ok(self
                .clone()
                .update_value(ValueKind::MapLike(object.clone().set(
                    MapKey::Implicit(MapKeyImplicit::new()),
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
            AccessKind::ArrayAccessImplicit | AccessKind::ArrayAccessExplicit { .. } => Type::Array,
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
            EvaluateError::MultilineStringNotStartingWithNewline { span } => [Level::Error
                .span(span.byte_range())
                .label("The content of a multiline string should start with a newline")]
            .into_iter()
            .collect_vec(),
            EvaluateError::MultilineStringNotEndingWithNewline { span } => [Level::Error
                .span(span.byte_range())
                .label("The content of a multiline string should end with a newline")]
            .into_iter()
            .collect_vec(),
        }
    }

    fn title(&self) -> &'static str {
        match self {
            EvaluateError::TypeMismatch(_) => "Type Mismatch",
            EvaluateError::DuplicateAssignment { .. } => "Duplicate Assignment",
            EvaluateError::StringUnescapeError { .. } => "String Unsecape Error",
            EvaluateError::MultilineStringNotStartingWithNewline { .. }
            | EvaluateError::MultilineStringNotEndingWithNewline { .. } => {
                "Incorrect multi-line string format"
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum EvaluateError {
    TypeMismatch(Box<TypeMismatch>),
    DuplicateAssignment {
        previously_assigned_at: Span,
        now_assigned_again_at: Span,
    },
    StringUnescapeError {
        span: Span,
        error: String,
    },
    MultilineStringNotStartingWithNewline {
        span: Span,
    },
    MultilineStringNotEndingWithNewline {
        span: Span,
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
        crate::parser::ValueKind::String(string) => ValueScalarKind::String(string),
        crate::parser::ValueKind::Integer(integer) => ValueScalarKind::Integer(integer),
        crate::parser::ValueKind::Decimal(decimal) => ValueScalarKind::Number(decimal),
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
