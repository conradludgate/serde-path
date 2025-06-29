use serde::de;
use serde_json::{Map, Value, value::RawValue};

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
    List(MultiVec<JsonField>),
    Map(MultiMap<JsonField>),
}

impl<'de, S> FilterChain<'de, S> for JsonField
where
    S: de::DeserializeSeed<'de, Value = Value> + Clone,
{
    type Value = Value;
    fn filter<D>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self {
            JsonField::Index(JsonFieldIndex::List(filter)) => filter.filter(seed, deserializer),
            JsonField::Index(JsonFieldIndex::Map(filter)) => filter.filter(seed, deserializer),
            JsonField::List(filter) => Ok(Value::Array(filter.filter(seed, deserializer)?)),
            JsonField::Map(filter) => {
                let raw = <&'de RawValue as de::Deserialize>::deserialize(deserializer)?;

                let mut map = Map::with_capacity(filter.0.len());
                for (key, filter) in filter.0 {
                    let value = filter
                        .filter(seed.clone(), raw)
                        .map_err(de::Error::custom)?;
                    map.insert(key, value);
                }
                Ok(Value::Object(map))
            }
        }
    }
}

// #[macro_use]
// macro_rules! json_path {
//     ([$($tt:tt)*]) => {
//         let fields: std::vec::Vec::<$crate::json::JsonField> = vec![
//             $crate::json::JsonField::List()
//         ];
//         $crate::Iter(.into_iter())
//     };
//     (@$($tt:tt)*) => {
//         let fields: std::vec::Vec::<$crate::json::JsonField> = crate::json::json_path!();
//         $crate::Iter(.into_iter())
//     };
// }

// #[cfg(test)]
// mod tests {
//     use std::{collections::HashMap, marker::PhantomData};

//     use serde_json::{json, value::RawValue};

//     use crate::{FilterChain, MultiVec, hlist, multi::MultiMap};

//     fn extract_json_path<'de, F, T>(
//         json: &'de str,
//         filter: F,
//     ) -> Result<F::Value, serde_json::Error>
//     where
//         T: serde::Deserialize<'de>,
//         F: FilterChain<'de, PhantomData<T>>,
//     {
//         filter.filter(
//             PhantomData::<T>,
//             &mut serde_json::Deserializer::from_str(json),
//         )
//     }

//     #[test]
//     fn list() {
//         let json = json!({
//             "a": 1,
//             "b": {
//                 "c": [2, 3, 4],
//                 "d": [5]
//             },
//             "e": 6,
//         })
//         .to_string();

//         let path = json_path!(@.a[0]);

//         let fields: Vec<&RawValue> = extract_json_path(
//             &json,
//             hlist!["b", MultiVec(vec![hlist!["c", 1], hlist!["d", 0]])],
//         )
//         .unwrap();

//         assert_eq!(fields.len(), 2);
//         assert_eq!(fields[0].get(), "3");
//         assert_eq!(fields[1].get(), "5");
//     }
// }
