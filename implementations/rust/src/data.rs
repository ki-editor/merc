use indexmap::IndexMap;
use rust_decimal::Decimal;

pub enum Value {
    Array(Vec<Value>),
    Tuple(Vec<Value>),
    Map(IndexMap<String, Value>),
    Object(IndexMap<String, Value>),
    String(String),
    Integer(isize),
    Decimal(Decimal),
    Comment(String),
}
