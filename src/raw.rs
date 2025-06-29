use std::{marker::PhantomData, ops::Deref};

use serde::{Deserializer, de};
use serde_json::value::RawValue;

use crate::borrow::Unborrow;

pub trait RawDeserializeSeed<'de> {
    type Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de> + Clone;
}

pub struct WithRawValue<F>(pub F);

impl<'de, F> de::DeserializeSeed<'de> for WithRawValue<F>
where
    F: RawDeserializeSeed<'de>,
{
    type Value = F::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(TOKEN, self)
    }
}

impl<'de, F> de::Visitor<'de> for WithRawValue<F>
where
    F: RawDeserializeSeed<'de>,
{
    type Value = F::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "any valid JSON value")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let value = map.next_key::<RawKey>()?;
        if value.is_none() {
            return Err(de::Error::invalid_type(de::Unexpected::Map, &self));
        }
        map.next_value_seed(WithRawValue2(self.0))
    }
}

struct WithRawValue2<F>(F);

impl<'de, F> de::DeserializeSeed<'de> for WithRawValue2<F>
where
    F: RawDeserializeSeed<'de>,
{
    type Value = F::Value;
    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de, F> de::Visitor<'de> for WithRawValue2<F>
where
    F: RawDeserializeSeed<'de>,
{
    type Value = F::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("raw value")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        self.0
            .deserialize(Unborrow(cast(s), PhantomData))
            .map_err(E::custom)
    }

    fn visit_borrowed_str<E: de::Error>(self, s: &'de str) -> Result<Self::Value, E> {
        self.0.deserialize(cast(s)).map_err(E::custom)
    }
}

fn cast(s: &str) -> &RawValue {
    // Safety: this is not sound since RawValue layout is not guaranteed.
    // This however works for now.
    unsafe { std::mem::transmute::<&str, &RawValue>(s) }
}
fn cast_box(s: Box<str>) -> Box<RawValue> {
    // Safety: this is not sound since RawValue layout is not guaranteed.
    // This however works for now.
    unsafe { std::mem::transmute::<Box<str>, Box<RawValue>>(s) }
}

// technically not stable api from serde_json.
// i don't really care.
const TOKEN: &str = "$serde_json::private::RawValue";
struct RawKey;

impl<'de> de::Deserialize<'de> for RawKey {
    fn deserialize<D>(deserializer: D) -> Result<RawKey, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct FieldVisitor;

        impl<'de> de::Visitor<'de> for FieldVisitor {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("raw value")
            }

            fn visit_str<E>(self, s: &str) -> Result<(), E>
            where
                E: de::Error,
            {
                if s == TOKEN {
                    Ok(())
                } else {
                    Err(de::Error::custom("unexpected raw value"))
                }
            }
        }

        deserializer.deserialize_identifier(FieldVisitor)?;
        Ok(RawKey)
    }
}

pub enum RawValue2<'de> {
    Borrowed(&'de RawValue),
    Owned(Box<RawValue>),
}

impl Deref for RawValue2<'_> {
    type Target = RawValue;

    fn deref(&self) -> &Self::Target {
        match self {
            RawValue2::Borrowed(v) => v,
            RawValue2::Owned(v) => v,
        }
    }
}

impl<'de> de::Deserialize<'de> for RawValue2<'de> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_newtype_struct(TOKEN, RawValue2Visitor)
    }
}

struct RawValue2Visitor;
impl<'de> de::Visitor<'de> for RawValue2Visitor {
    type Value = RawValue2<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "any valid JSON value")
    }

    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let value = map.next_key::<RawKey>()?;
        if value.is_none() {
            return Err(de::Error::invalid_type(de::Unexpected::Map, &self));
        }
        map.next_value_seed(RawValue2Visitor2)
    }
}

struct RawValue2Visitor2;

impl<'de> de::DeserializeSeed<'de> for RawValue2Visitor2 {
    type Value = RawValue2<'de>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

impl<'de> de::Visitor<'de> for RawValue2Visitor2 {
    type Value = RawValue2<'de>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("raw value")
    }

    fn visit_string<E: de::Error>(self, s: String) -> Result<Self::Value, E> {
        Ok(RawValue2::Owned(cast_box(s.into_boxed_str())))
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        Ok(RawValue2::Owned(cast(s).to_owned()))
    }

    fn visit_borrowed_str<E: de::Error>(self, s: &'de str) -> Result<Self::Value, E> {
        Ok(RawValue2::Borrowed(cast(s)))
    }
}
