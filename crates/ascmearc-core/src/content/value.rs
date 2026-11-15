//! A parsed content document as a plain tree, and the one serde deserializer the content types
//! are read through from it (#1720).
//!
//! A TOML file is read straight into its types with `toml` (its errors point at the line); the
//! packed form of the built-in content (`content/pack.rs`, the `packed` feature) is decoded into
//! a [`Value`] and read from that. The loaders that edit a document before reading it (a
//! generated grimoire's object table, a mimic's creature table) build a [`Table`] the same way.
//! The deserializer behaves as `toml`'s does for every shape content uses: integers are `i64`,
//! an option is `Some` whenever the key is there, an enum is a string (unit variant) or a table of
//! one key, and a table's keys come in sorted order (`toml` keeps no document order either).

use serde::de::{self, DeserializeSeed, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;
use std::fmt;

/// A table: keys in sorted order, as `toml::Table`.
pub type Table = BTreeMap<String, Value>;

/// A value of a content document.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Array(Vec<Value>),
    Table(Table),
}

impl Value {
    /// The value under `key`, if this is a table that has it.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Table(t) => t.get(key),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    fn unexpected(&self) -> de::Unexpected<'_> {
        match self {
            Value::String(s) => de::Unexpected::Str(s),
            Value::Integer(i) => de::Unexpected::Signed(*i),
            Value::Float(f) => de::Unexpected::Float(*f),
            Value::Boolean(b) => de::Unexpected::Bool(*b),
            Value::Array(_) => de::Unexpected::Seq,
            Value::Table(_) => de::Unexpected::Map,
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Value::Integer(i)
    }
}

/// Why a [`Value`] did not deserialize: serde's own words for it.
#[derive(Clone, Debug, PartialEq)]
pub struct Error(String);

impl Error {
    pub(super) fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self(msg.to_string())
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(ValueVisitor)
    }
}

struct ValueVisitor;

impl<'de> Visitor<'de> for ValueVisitor {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("any value")
    }

    fn visit_bool<E>(self, b: bool) -> Result<Value, E> {
        Ok(Value::Boolean(b))
    }

    fn visit_i64<E>(self, i: i64) -> Result<Value, E> {
        Ok(Value::Integer(i))
    }

    fn visit_u64<E: de::Error>(self, u: u64) -> Result<Value, E> {
        i64::try_from(u)
            .map(Value::Integer)
            .map_err(|_| E::custom("integer out of range"))
    }

    fn visit_f64<E>(self, f: f64) -> Result<Value, E> {
        Ok(Value::Float(f))
    }

    fn visit_str<E>(self, s: &str) -> Result<Value, E> {
        Ok(Value::String(s.to_string()))
    }

    fn visit_string<E>(self, s: String) -> Result<Value, E> {
        Ok(Value::String(s))
    }

    fn visit_some<D: Deserializer<'de>>(self, d: D) -> Result<Value, D::Error> {
        Value::deserialize(d)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut out = Vec::new();
        while let Some(v) = seq.next_element()? {
            out.push(v);
        }
        Ok(Value::Array(out))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut out = Table::new();
        while let Some((k, v)) = map.next_entry::<String, Value>()? {
            out.insert(k, v);
        }
        Ok(Value::Table(out))
    }
}

impl<'de> IntoDeserializer<'de, Error> for Value {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self {
        self
    }
}

impl<'de> Deserializer<'de> for Value {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self {
            Value::String(s) => visitor.visit_string(s),
            Value::Integer(i) => visitor.visit_i64(i),
            Value::Float(f) => visitor.visit_f64(f),
            Value::Boolean(b) => visitor.visit_bool(b),
            Value::Array(a) => {
                let len = a.len();
                let mut seq = SeqDeserializer(a.into_iter());
                let out = visitor.visit_seq(&mut seq)?;
                match seq.0.len() {
                    0 => Ok(out),
                    left => Err(de::Error::invalid_length(len - left, &"fewer elements")),
                }
            }
            Value::Table(t) => visitor.visit_map(MapDeserializer {
                iter: t.into_iter(),
                value: None,
            }),
        }
    }

    /// No value is none in TOML: a key that is there is always `Some`.
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        visitor.visit_some(self)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    /// A unit variant is its name; any other is a table of one key, the variant, holding it.
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self {
            Value::String(s) => visitor.visit_enum(s.into_deserializer()),
            Value::Table(t) if t.len() == 1 => {
                let (variant, value) = t.into_iter().next().expect("one entry");
                visitor.visit_enum(EnumDeserializer { variant, value })
            }
            Value::Table(_) => Err(de::Error::invalid_length(0, &"a table of one key")),
            other => Err(de::Error::invalid_type(
                other.unexpected(),
                &"a string or a table",
            )),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf unit
        unit_struct seq tuple tuple_struct map struct identifier ignored_any
    }
}

struct SeqDeserializer(std::vec::IntoIter<Value>);

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T: DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Error> {
        self.0.next().map(|v| seed.deserialize(v)).transpose()
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.0.len())
    }
}

struct MapDeserializer {
    iter: std::collections::btree_map::IntoIter<String, Value>,
    value: Option<Value>,
}

impl<'de> MapAccess<'de> for MapDeserializer {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some((key, value)) = self.iter.next() else {
            return Ok(None);
        };
        self.value = Some(value);
        seed.deserialize(Value::String(key)).map(Some)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value, Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| Error::new("value before key"))?;
        seed.deserialize(value)
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.iter.len())
    }
}

struct EnumDeserializer {
    variant: String,
    value: Value,
}

impl<'de> de::EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = Value;

    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Value), Error> {
        let variant = seed.deserialize(Value::String(self.variant))?;
        Ok((variant, self.value))
    }
}

impl<'de> de::VariantAccess<'de> for Value {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self {
            Value::Table(t) if t.is_empty() => Ok(()),
            other => Err(de::Error::invalid_type(
                other.unexpected(),
                &"a unit variant",
            )),
        }
    }

    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value, Error> {
        seed.deserialize(self)
    }

    fn tuple_variant<V: Visitor<'de>>(self, _len: usize, visitor: V) -> Result<V::Value, Error> {
        self.deserialize_any(visitor)
    }

    fn struct_variant<V: Visitor<'de>>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        self.deserialize_any(visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct Shape {
        n: u8,
        maybe: Option<String>,
        list: Vec<f32>,
        kind: Kind,
        #[serde(default)]
        nested: BTreeMap<String, i32>,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "snake_case")]
    enum Kind {
        Plain,
        Wrapped(i32),
    }

    fn parse(text: &str) -> Result<Shape, Error> {
        let value: Value = toml::from_str(text).expect("toml");
        Shape::deserialize(value)
    }

    #[test]
    fn a_value_reads_as_toml_reads_the_text() {
        for text in [
            "n = 3\nlist = [1, 2.5]\nkind = \"plain\"\n",
            "n = 3\nmaybe = \"x\"\nlist = []\nkind = { wrapped = 4 }\nnested = { a = 1 }\n",
        ] {
            let direct: Shape = toml::from_str(text).expect("toml");
            assert_eq!(parse(text).expect(text), direct);
        }
    }

    #[test]
    fn a_bad_value_is_refused_in_serdes_words() {
        let e = parse("n = 300\nlist = []\nkind = \"plain\"\n").unwrap_err();
        assert!(e.to_string().contains("expected u8"), "{e}");
        let e = parse("n = 1\nlist = []\nkind = \"plain\"\nextra = 1\n").unwrap_err();
        assert!(e.to_string().contains("unknown field `extra`"), "{e}");
        let e = parse("n = 1\nlist = []\nkind = \"round\"\n").unwrap_err();
        assert!(e.to_string().contains("unknown variant `round`"), "{e}");
    }
}
