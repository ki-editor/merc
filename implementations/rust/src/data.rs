use indexmap::IndexMap;
use itertools::Itertools;
use rust_decimal::Decimal;

use crate::parser::{Access, AccessKind, Parsed, Span, Statement, ValueKind};

#[derive(Clone, Debug)]
pub enum Value {
    ArrayLike(ArrayLike),
    MapLike(MapLike),
    String(String),
    Integer(isize),
    Decimal(Decimal),
    Null,
    Boolean(bool),
    Uninitialized,
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

    fn push_new(mut self, tail: &[Access], value: Value) -> Result<ArrayLike, EvaluationError> {
        let new_element = Value::Uninitialized.set(tail, value)?;
        self.array.push(new_element);
        Ok(self)
    }

    fn set_last(mut self, tail: &[Access], value: Value) -> Result<ArrayLike, EvaluationError> {
        if let Some(last) = self.array.pop() {
            self.array.push(last.set(tail, value)?);
            Ok(self)
        } else {
            Err(EvaluationError {
                span: todo!(),
                kind: ErrorKind::LastArrayElementNotFound,
            })
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

    fn set(self, key: &str, tail: &[Access], value: Value) -> Result<Self, EvaluationError> {
        let mut map = self.map;
        let map = if let Some(object_value) = map.shift_remove(key) {
            map.insert(key.to_string(), object_value.set(tail, value)?);
            map
        } else {
            map.insert(key.to_string(), Value::Uninitialized.set(tail, value)?);
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
    fn new_map_like(kind: MapKind, key: String, value: Value) -> Value {
        Value::MapLike(MapLike {
            kind,
            map: {
                let mut map = IndexMap::new();
                map.insert(key, value);
                map
            },
        })
    }

    fn update(self, entry: crate::parser::Entry) -> Result<Value, EvaluationError> {
        self.set(
            &entry.accesses.into_iter().collect_vec(),
            evaluate_value(entry.value),
        )
    }

    fn set(self, accesses: &[Access], value: Value) -> Result<Value, EvaluationError> {
        let Some((head, tail)) = accesses.split_first() else {
            return Ok(value);
        };

        let span = head.span.clone();

        match (self, &head.kind) {
            (Value::Uninitialized, AccessKind::MapAccess { .. }) => {
                Value::MapLike(MapLike::new(MapKind::Map)).set(accesses, value)
            }
            (Value::Uninitialized, AccessKind::ObjectAccess { .. }) => {
                Value::MapLike(MapLike::new(MapKind::Object)).set(accesses, value)
            }
            (Value::Uninitialized, AccessKind::ArrayAccessNew) => {
                Value::ArrayLike(ArrayLike::new(ArrayKind::Array)).set(accesses, value)
            }
            (Value::Uninitialized, AccessKind::TupleAccessNew) => {
                Value::ArrayLike(ArrayLike::new(ArrayKind::Tuple)).set(accesses, value)
            }
            (Value::ArrayLike(array), AccessKind::ArrayAccessNew) => {
                Ok(Value::ArrayLike(array.push_new(tail, value)?))
            }
            (Value::ArrayLike(array), AccessKind::ArrayAccessLast) => {
                Ok(Value::ArrayLike(array.set_last(tail, value)?))
            }
            (Value::ArrayLike(array), AccessKind::TupleAccessNew) => {
                Ok(Value::ArrayLike(array.push_new(tail, value)?))
            }
            (Value::ArrayLike(array), AccessKind::TupleAccessLast) => {
                Ok(Value::ArrayLike(array.set_last(tail, value)?))
            }
            (
                Value::MapLike(
                    object @ MapLike {
                        kind: MapKind::Object,
                        ..
                    },
                ),
                AccessKind::ObjectAccess { key },
            ) => Ok(Value::MapLike(object.set(key, tail, value)?)),
            (
                Value::MapLike(
                    object @ MapLike {
                        kind: MapKind::Map, ..
                    },
                ),
                AccessKind::MapAccess { key },
            ) => Ok(Value::MapLike(object.set(key, tail, value)?)),
            (expected_value, actual_access) => Err(EvaluationError {
                kind: ErrorKind::TypeMismatch {
                    expected_type: expected_value.typ(),
                    actual_type: actual_access.typ(),
                },
                span,
            }),
        }
    }

    fn typ(&self) -> Type {
        match self {
            Value::ArrayLike(ArrayLike {
                kind: ArrayKind::Array,
                ..
            }) => Type::Array,
            Value::ArrayLike(ArrayLike {
                kind: ArrayKind::Tuple,
                ..
            }) => Type::Tuple,
            Value::MapLike(MapLike {
                kind: MapKind::Object,
                ..
            }) => Type::Object,
            Value::MapLike(MapLike {
                kind: MapKind::Map, ..
            }) => Type::Map,
            Value::String(_) => Type::String,
            Value::Integer(_) => Type::Integer,
            Value::Decimal(_) => Type::Decimal,
            Value::Null => Type::Null,
            Value::Boolean(_) => Type::Boolean,
            Value::Uninitialized => Type::Uninitialized,
        }
    }

    pub(crate) fn into_json(self) -> serde_json::Value {
        match self {
            Value::ArrayLike(array_like) => array_like.into_json(),
            Value::MapLike(map_like) => map_like.into_json(),
            Value::String(string) => serde_json::Value::String(string),
            Value::Integer(integer) => serde_json::Value::Number(integer.into()),
            Value::Decimal(decimal) => str::parse(&decimal.to_string())
                .map(serde_json::Value::Number)
                .unwrap_or_else(|_| serde_json::Value::String(decimal.to_string())),
            Value::Null => serde_json::Value::Null,
            Value::Boolean(boolean) => serde_json::Value::Bool(boolean),
            Value::Uninitialized => unreachable!(),
        }
    }
}

impl Access {
    pub(crate) fn init_value(
        &self,
        value: crate::data::Value,
    ) -> Result<crate::data::Value, crate::data::EvaluationError> {
        match &self.kind {
            AccessKind::ObjectAccess { key } => {
                Ok(Value::new_map_like(MapKind::Object, key.to_string(), value))
            }
            AccessKind::MapAccess { key } => {
                Ok(Value::new_map_like(MapKind::Map, key.to_string(), value))
            }
            AccessKind::ArrayAccessNew => todo!(),
            AccessKind::ArrayAccessLast => todo!(),
            AccessKind::TupleAccessLast => todo!(),
            AccessKind::TupleAccessNew => todo!(),
        }
    }
}
impl AccessKind {
    pub(crate) fn typ(&self) -> crate::data::Type {
        match self {
            AccessKind::ObjectAccess { key } => Type::Object,
            AccessKind::MapAccess { key } => Type::Map,
            AccessKind::ArrayAccessNew | AccessKind::ArrayAccessLast => Type::Array,
            AccessKind::TupleAccessLast | AccessKind::TupleAccessNew => Type::Tuple,
        }
    }
}

fn get_value(accesses: &[crate::parser::Access], value: Value) -> Result<Value, EvaluationError> {
    match accesses.split_first() {
        Some((head, tail)) => head.init_value(get_value(tail, value)?),
        None => Ok(value),
    }
}

#[derive(Debug)]
pub(crate) struct EvaluationError {
    span: Span,
    kind: ErrorKind,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    TypeMismatch {
        expected_type: Type,
        actual_type: Type,
    },
    LastArrayElementNotFound,
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
    Comment,
    Null,
    Boolean,
    Uninitialized,
}

pub(crate) fn evaluate(parsed: Parsed) -> Result<Value, EvaluationError> {
    let result = Value::Uninitialized;
    parsed
        .into_entries()
        .into_iter()
        .try_fold(result, |result, entry| result.update(entry))
}

fn evaluate_entry(
    mut result: Option<Value>,
    entry: crate::parser::Entry,
) -> Result<Option<Value>, EvaluationError> {
    let (init, last) = {
        let head = entry.accesses.head;
        let tail = entry.accesses.tail;
        match tail.split_last() {
            Some((last, init)) => (
                Some(head).into_iter().chain(init.to_vec()).collect_vec(),
                last.clone(),
            ),
            None => (vec![], head.clone()),
        }
    };
    let first_value = construct_value(last, evaluate_value(entry.value))?;
    Ok(result.clone())
}

fn construct_value(last: crate::parser::Access, value: Value) -> Result<Value, EvaluationError> {
    todo!()
}

fn evaluate_value(value: crate::parser::EntryValue) -> Value {
    match value.kind {
        ValueKind::MultilineString(string) | ValueKind::String(string) => Value::String(string),
        ValueKind::Integer(integer) => todo!("integer = {integer}"),
        ValueKind::Decimal(decimal) => Value::Decimal(decimal),
        ValueKind::Boolean(boolean) => Value::Boolean(boolean),
        ValueKind::Null => Value::Null,
    }
}
