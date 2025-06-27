use std::marker::PhantomData;

use serde::de;

#[derive(Debug)]
pub enum Field<'a> {
    Index(usize),
    Key(&'a str),
}

pub struct Path<'a, S> {
    path: &'a [Field<'a>],
    seed: S,
}

impl<'a, S> Path<'a, S> {
    pub fn new(path: &'a [Field<'a>], seed: S) -> Self {
        Self { path, seed }
    }
}

impl<'de, S> de::DeserializeSeed<'de> for Path<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.path {
            [] => self.seed.deserialize(deserializer),
            [Field::Key(_), ..] => deserializer.deserialize_map(PathVisitor(self)),
            [Field::Index(_), ..] => deserializer.deserialize_seq(PathVisitor(self)),
        }
    }
}

struct PathVisitor<'a, T>(Path<'a, T>);

impl<'de, S> de::Visitor<'de> for PathVisitor<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "path {:?}", self.0.path)
    }

    #[inline]
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let [Field::Index(head), tail @ ..] = self.0.path else {
            panic!("path should always have an index value")
        };
        let seed = Path {
            path: tail,
            seed: self.0.seed,
        };

        let mut i = 0;
        let value = loop {
            if i == *head {
                break seq.next_element_seed(seed)?;
            }

            if seq.next_element::<de::IgnoredAny>()?.is_none() {
                break None;
            }
            i += 1;
        };

        let value =
            value.ok_or_else(|| de::Error::custom(format_args!("missing field `{head:?}`")))?;

        while seq.next_element::<de::IgnoredAny>()?.is_some() {}
        Ok(value)
    }

    #[inline]
    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let [Field::Key(head), tail @ ..] = self.0.path else {
            panic!("path should always have a key value")
        };
        let visitor = FieldVisitor(head);
        let seed = Path {
            path: tail,
            seed: self.0.seed,
        };

        let value = loop {
            let Some(found) = map.next_key_seed(visitor)? else {
                break None;
            };

            if found {
                break Some(map.next_value_seed(seed)?);
            }

            map.next_value::<de::IgnoredAny>()?;
        };

        let value =
            value.ok_or_else(|| de::Error::custom(format_args!("missing field `{head:?}`")))?;

        while let Some((found, _)) = map.next_entry_seed(visitor, PhantomData::<de::IgnoredAny>)? {
            if found {
                return Err(de::Error::custom(format_args!("duplicate field `{head}`")));
            }
        }

        Ok(value)
    }
}

#[derive(Clone, Copy)]
struct FieldVisitor<'a>(&'a str);

impl<'de> de::Visitor<'de> for FieldVisitor<'_> {
    type Value = bool;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Formatter::write_str(formatter, "field identifier")
    }

    #[inline]
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_bytes(v.as_bytes())
    }

    #[inline]
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v == self.0.as_bytes())
    }
}

impl<'de> de::DeserializeSeed<'de> for FieldVisitor<'_> {
    type Value = bool;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}

#[cfg(test)]
mod tests {
    use serde::de::DeserializeSeed;
    use serde_json::{json, value::RawValue};

    use crate::{Field, Path};

    fn extract_json_path<'de, T>(json: &'de str, path: &[Field]) -> Result<T, serde_json::Error>
    where
        T: serde::Deserialize<'de>,
    {
        Path::new(path, std::marker::PhantomData::<T>)
            .deserialize(&mut serde_json::Deserializer::from_str(json))
    }

    #[test]
    fn it_works() {
        let json = json!({ "a": 1, "b": {"c": [2, 3, 4]}, "d": 5}).to_string();
        let field: &RawValue =
            extract_json_path(&json, &[Field::Key("b"), Field::Key("c"), Field::Index(1)]).unwrap();
        assert_eq!(field.get(), "3")
    }
}
