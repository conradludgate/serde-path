use std::{marker::PhantomData, vec::IntoIter};

use serde::{de, forward_to_deserialize_any};
use serde_json::{Value, value::RawValue};

use crate::{FilterChain, MultiMap, MultiVec};

#[derive(Debug, Clone)]
pub enum JsonFieldIndex {
    List(usize),
    Map(String),
}

impl From<&str> for JsonFieldIndex {
    fn from(value: &str) -> Self {
        JsonFieldIndex::Map(value.to_string())
    }
}

impl From<usize> for JsonFieldIndex {
    fn from(value: usize) -> Self {
        JsonFieldIndex::List(value)
    }
}

#[derive(Debug, Clone)]
pub enum JsonField {
    Index(JsonFieldIndex),
    List(MultiVec<JsonPath>),
    Map(MultiMap<JsonPath>),
}

#[derive(Debug, Clone)]
pub struct JsonPath(pub IntoIter<JsonField>);

impl JsonPath {
    fn filter_inner<'de, S>(
        mut self,
        seed: S,
        mut raw: &'de RawValue,
    ) -> Result<S::Value, serde_json::Error>
    where
        S: de::DeserializeSeed<'de>,
    {
        for filter in self.0.by_ref() {
            match filter {
                JsonField::Index(JsonFieldIndex::List(filter)) => {
                    raw = filter.filter(PhantomData, raw)?;
                }
                JsonField::Index(JsonFieldIndex::Map(filter)) => {
                    raw = filter.filter(PhantomData, raw)?;
                }
                JsonField::List(filter) => {
                    let mut list = Vec::with_capacity(filter.0.len());
                    for filter in filter.0 {
                        list.push(filter.filter(PhantomData, raw)?);
                    }
                    return seed.deserialize(JsonList {
                        path: self,
                        iter: list.into_iter(),
                    });
                }
                JsonField::Map(filter) => {
                    let mut list = Vec::with_capacity(filter.0.len());
                    for (key, filter) in filter.0 {
                        list.push((key, filter.filter(PhantomData, raw)?));
                    }
                    return seed.deserialize(JsonMap {
                        path: self,
                        raw: None,
                        iter: list.into_iter(),
                    });
                }
            }
        }

        seed.deserialize(raw)
    }
}

impl<'de> FilterChain<'de> for JsonPath {
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        let raw = <&'de RawValue as de::Deserialize>::deserialize(deserializer)?;
        self.filter_inner(seed, raw).map_err(de::Error::custom)
    }
}

struct JsonList<'de> {
    path: JsonPath,
    iter: IntoIter<&'de RawValue>,
}

impl<'de> de::Deserializer<'de> for JsonList<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_seq(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::SeqAccess<'de> for JsonList<'de> {
    type Error = serde_json::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(raw) = self.iter.next() else {
            return Ok(None);
        };
        self.path.clone().filter_inner(seed, raw).map(Some)
    }
}

struct JsonMap<'de> {
    path: JsonPath,
    raw: Option<&'de RawValue>,
    iter: IntoIter<(String, &'de RawValue)>,
}

impl<'de> de::Deserializer<'de> for JsonMap<'de> {
    type Error = serde_json::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: de::Visitor<'de>,
    {
        visitor.visit_map(self)
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

impl<'de> de::MapAccess<'de> for JsonMap<'de> {
    type Error = serde_json::Error;

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
        V: de::DeserializeSeed<'de>,
    {
        let Some((key, raw)) = self.iter.next() else {
            return Ok(None);
        };
        let key = kseed.deserialize(Value::String(key))?;
        let value = self.path.clone().filter_inner(vseed, raw)?;
        Ok(Some((key, value)))
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some((key, raw)) = self.iter.next() else {
            return Ok(None);
        };
        self.raw = Some(raw);
        seed.deserialize(Value::String(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.path.clone().filter_inner(seed, self.raw.unwrap())
    }
}

// #[macro_export]
// macro_rules! json_path {
//     ($($tt:tt)*) => { $crate::json::JsonPath($crate::json_path_inner!(vec![]; $($tt)*).into_iter()) }
// }

// #[doc(hidden)]
// #[macro_export]
// macro_rules! json_path_inner {
//     ($fields:expr; [$($tt:tt)*]) => {{
//         let mut fields = $fields;
//         fields.push($crate::json::JsonField::List($crate::MultiVec($crate::json_path_inner!(vec![]$($tt)*))));
//         fields
//     }};
//     ($fields:expr; @[$lit:literal] $($tt:tt)*) => {{
//         let mut fields = $fields;
//         fields.push($crate::json::JsonField::Index($lit.into()));
//         $crate::json_path_inner!(fields; @$($tt)*)
//     }};
//     ($fields:expr;) => { $fields }
// }

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use serde_json::{Serializer, Value, json};

    use crate::{
        FilterChain,
        json::{JsonField, JsonPath},
        json_ser::JsonSer,
    };

    fn extract_json_path<'de, F, T>(json: &'de str, filter: F) -> Result<T, serde_json::Error>
    where
        T: serde::Deserialize<'de>,
        F: FilterChain<'de>,
    {
        filter.filter(
            PhantomData::<T>,
            &mut serde_json::Deserializer::from_str(json),
        )
    }

    #[test]
    fn list() {
        let json = json!({
            "a": 1,
            "b": {
                "c": [2, 3, 4],
                "d": [5]
            },
            "e": 6,
        })
        .to_string();

        // let path = json_path!(@["b"][@["c"][1], @["d"][0]]);
        let path = JsonPath(
            vec![
                JsonField::Index("b".into()),
                JsonField::List(crate::MultiVec(vec![
                    JsonPath(
                        vec![JsonField::Index("c".into()), JsonField::Index(1.into())].into_iter(),
                    ),
                    JsonPath(
                        vec![JsonField::Index("d".into()), JsonField::Index(0.into())].into_iter(),
                    ),
                ])),
            ]
            .into_iter(),
        );

        let fields: Value = extract_json_path(&json, path).unwrap();
        assert_eq!(fields, json!([3, 5]));
    }

    #[test]
    fn list_streaming() {
        let json = json!({
            "a": 1,
            "b": {
                "c": [2, 3, 4],
                "d": [5]
            },
            "e": 6,
        })
        .to_string();

        // let path = json_path!(@["b"][@["c"][1], @["d"][0]]);
        let path = JsonPath(
            vec![
                JsonField::Index("b".into()),
                JsonField::List(crate::MultiVec(vec![
                    JsonPath(
                        vec![JsonField::Index("c".into()), JsonField::Index(1.into())].into_iter(),
                    ),
                    JsonPath(
                        vec![JsonField::Index("d".into()), JsonField::Index(0.into())].into_iter(),
                    ),
                ])),
            ]
            .into_iter(),
        );

        let ser = Serializer::pretty(Vec::new());
        let output = path
            .filter(JsonSer(ser), &mut serde_json::Deserializer::from_str(&json))
            .unwrap();
        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, "[\n  3,\n  5\n]");
    }
}
