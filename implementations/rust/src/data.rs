use annotate_snippets::{Annotation, Level};
use indexmap::IndexMap;
use itertools::Itertools;
use rust_decimal::Decimal;

use crate::parser::{Access, AccessKind, Parsed, Span, ValueKind};

#[derive(Clone, Debug)]
pub(crate) struct Value {
    typ: ValueType,
    inferred_at: Span,
}

#[derive(Clone, Debug)]
enum ValueType {
    String(String),
    Integer(isize),
    Decimal(Decimal),
    Null,
    Boolean(bool),
    ArrayLike(ArrayLike),
    MapLike(MapLike),
    Uninitialized,
}

impl ValueType {
    fn typ(&self) -> Type {
        match self {
            ValueType::ArrayLike(ArrayLike {
                kind: ArrayKind::Array,
                ..
            }) => Type::Array,
            ValueType::ArrayLike(ArrayLike {
                kind: ArrayKind::Tuple,
                ..
            }) => Type::Tuple,
            ValueType::MapLike(MapLike {
                kind: MapKind::Object,
                ..
            }) => Type::Object,
            ValueType::MapLike(MapLike {
                kind: MapKind::Map, ..
            }) => Type::Map,
            ValueType::Uninitialized => Type::Uninitialized,
            ValueType::String(_) => Type::String,
            ValueType::Integer(_) => Type::Integer,
            ValueType::Decimal(_) => Type::Decimal,
            ValueType::Null => Type::Null,
            ValueType::Boolean(_) => Type::Boolean,
        }
    }

    fn is_scalar(&self) -> bool {
        matches!(
            self,
            ValueType::String(_)
                | ValueType::Integer(_)
                | ValueType::Decimal(_)
                | ValueType::Null
                | ValueType::Boolean(_)
        )
    }

    fn into_json(self) -> serde_json::Value {
        match self {
            ValueType::ArrayLike(array_like) => array_like.into_json(),
            ValueType::MapLike(map_like) => map_like.into_json(),
            ValueType::String(string) => serde_json::Value::String(string),
            ValueType::Integer(integer) => serde_json::Value::Number(integer.into()),
            ValueType::Decimal(decimal) => str::parse(&decimal.to_string())
                .map(serde_json::Value::Number)
                .unwrap_or_else(|_| serde_json::Value::String(decimal.to_string())),
            ValueType::Null => serde_json::Value::Null,
            ValueType::Boolean(boolean) => serde_json::Value::Bool(boolean),
            ValueType::Uninitialized => unreachable!(),
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
            todo!()
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
    map: IndexMap<String, Value>,
}
impl MapLike {
    fn new(kind: MapKind) -> Self {
        Self {
            kind,
            map: Default::default(),
        }
    }

    fn set(self, key: &str, tail: &[Access], value: Value) -> Result<Self, EvaluateError> {
        let mut map = self.map;
        let map = if let Some(current_value) = map.shift_remove(key) {
            if current_value.is_scalar() && value.is_scalar() {
                return Err(EvaluateError::DuplicateAssignment {
                    previously_assigned_at: current_value.inferred_at,
                    now_assigned_again_at: value.inferred_at,
                });
            }
            map.insert(key.to_string(), current_value.set(tail, value)?);
            map
        } else {
            map.insert(key.to_string(), Value::uninitialized().set(tail, value)?);
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
                .map(|(key, value)| (key, value.into_json()))
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
    fn update(self, entry: crate::parser::Entry) -> Result<Value, EvaluateError> {
        self.set(
            &entry.accesses.into_iter().collect_vec(),
            evaluate_value(entry.value),
        )
    }

    fn set(self, accesses: &[Access], value: Value) -> Result<Value, EvaluateError> {
        let Some((head, tail)) = accesses.split_first() else {
            return Ok(value);
        };

        let span = head.span.clone();

        match (&self.typ, &head.kind) {
            (ValueType::Uninitialized, AccessKind::MapAccess { .. }) => Value {
                inferred_at: span,
                typ: ValueType::MapLike(MapLike::new(MapKind::Map)),
            }
            .set(accesses, value),
            (ValueType::Uninitialized, AccessKind::ObjectAccess { .. }) => Value {
                inferred_at: span,
                typ: ValueType::MapLike(MapLike::new(MapKind::Object)),
            }
            .set(accesses, value),
            (ValueType::Uninitialized, AccessKind::ArrayAccessNew) => Value {
                inferred_at: span,
                typ: ValueType::ArrayLike(ArrayLike::new(ArrayKind::Array)),
            }
            .set(accesses, value),
            (ValueType::Uninitialized, AccessKind::TupleAccessNew) => Value {
                inferred_at: span,
                typ: ValueType::ArrayLike(ArrayLike::new(ArrayKind::Tuple)),
            }
            .set(accesses, value),
            (ValueType::Uninitialized, AccessKind::ArrayAccessLast) => {
                Err(EvaluateError::LastArrayElementNotFound { span })
            }
            (ValueType::ArrayLike(array), AccessKind::ArrayAccessNew) => Ok(self
                .clone()
                .update_value(ValueType::ArrayLike(array.clone().push_new(tail, value)?))),
            (ValueType::ArrayLike(array), AccessKind::ArrayAccessLast) => Ok(self
                .clone()
                .update_value(ValueType::ArrayLike(array.clone().set_last(tail, value)?))),
            (ValueType::ArrayLike(array), AccessKind::TupleAccessNew) => Ok(self
                .clone()
                .update_value(ValueType::ArrayLike(array.clone().push_new(tail, value)?))),
            (ValueType::ArrayLike(array), AccessKind::TupleAccessLast) => Ok(self
                .clone()
                .update_value(ValueType::ArrayLike(array.clone().set_last(tail, value)?))),
            (
                ValueType::MapLike(
                    object @ MapLike {
                        kind: MapKind::Object,
                        ..
                    },
                ),
                AccessKind::ObjectAccess { key },
            ) => Ok(self
                .clone()
                .update_value(ValueType::MapLike(object.clone().set(key, tail, value)?))),
            (
                ValueType::MapLike(
                    object @ MapLike {
                        kind: MapKind::Map, ..
                    },
                ),
                AccessKind::MapAccess { key },
            ) => Ok(self
                .clone()
                .update_value(ValueType::MapLike(object.clone().set(key, tail, value)?))),
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
        self.typ.into_json()
    }

    fn uninitialized() -> Value {
        Value {
            typ: ValueType::Uninitialized,
            inferred_at: Span::default(),
        }
    }

    fn update_value(self, typ: ValueType) -> Value {
        Value { typ, ..self }
    }

    fn is_scalar(&self) -> bool {
        self.typ.is_scalar()
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
        }
    }

    fn title(&self) -> &'static str {
        match self {
            EvaluateError::TypeMismatch(_) => "Type Mismatch",
            EvaluateError::LastArrayElementNotFound { .. } => "Last Array Element Not Found",
            EvaluateError::DuplicateAssignment { .. } => "Duplicate Assignment",
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
    Uninitialized,
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
            Type::Uninitialized => "Uninitialized",
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

fn evaluate_value(value: crate::parser::EntryValue) -> Value {
    let typ = match value.kind {
        ValueKind::MultilineString(string) | ValueKind::String(string) => ValueType::String(string),
        ValueKind::Integer(integer) => ValueType::Integer(integer),
        ValueKind::Decimal(decimal) => ValueType::Decimal(decimal),
        ValueKind::Boolean(boolean) => ValueType::Boolean(boolean),
        ValueKind::Null => ValueType::Null,
    };
    Value {
        typ,
        inferred_at: value.span,
    }
}
