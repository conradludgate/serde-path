use std::vec::IntoIter;

use serde::de::{
    self,
    value::{MapAccessDeserializer, SeqAccessDeserializer, StrDeserializer},
};
use serde_json::value::RawValue;

use crate::FilterChain;

#[derive(Debug, Clone)]
pub struct MultiVec<F>(pub Vec<F>);

impl<'de, F> FilterChain<'de> for MultiVec<F>
where
    F: FilterChain<'de>,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        let raw = <&'de RawValue as de::Deserialize>::deserialize(deserializer)?;
        seed.deserialize(SeqAccessDeserializer::new(MultiSeqAccess {
            raw,
            filters: self.0.into_iter(),
        }))
        .map_err(de::Error::custom)
    }
}

struct MultiSeqAccess<'de, F> {
    raw: &'de RawValue,
    filters: IntoIter<F>,
}

impl<'de, F> de::SeqAccess<'de> for MultiSeqAccess<'de, F>
where
    F: FilterChain<'de>,
{
    type Error = serde_json::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        let Some(filter) = self.filters.next() else {
            return Ok(None);
        };

        filter.filter(seed, self.raw).map(Some)
    }
}

#[derive(Debug, Clone)]
pub struct MultiMap<F>(pub Vec<(String, F)>);

impl<'de, F> FilterChain<'de> for MultiMap<F>
where
    F: FilterChain<'de>,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        let raw = <&'de RawValue as de::Deserialize>::deserialize(deserializer)?;
        seed.deserialize(MapAccessDeserializer::new(MultiMapAccess {
            raw,
            filter: None,
            filters: self.0.into_iter(),
        }))
        .map_err(de::Error::custom)
    }
}

struct MultiMapAccess<'de, F> {
    raw: &'de RawValue,
    filter: Option<F>,
    filters: IntoIter<(String, F)>,
}

impl<'de, F> de::MapAccess<'de> for MultiMapAccess<'de, F>
where
    F: FilterChain<'de>,
{
    type Error = serde_json::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        let Some((key, filter)) = self.filters.next() else {
            return Ok(None);
        };
        self.filter = Some(filter);
        seed.deserialize(StrDeserializer::new(&key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        self.filter.take().unwrap().filter(seed, self.raw)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, marker::PhantomData};

    use serde_json::{json, value::RawValue};

    use crate::{FilterChain, MultiVec, hlist, multi::MultiMap};

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

        let fields: Vec<&RawValue> = extract_json_path(
            &json,
            hlist!["b", MultiVec(vec![hlist!["c", 1_usize], hlist!["d", 0]])],
        )
        .unwrap();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].get(), "3");
        assert_eq!(fields[1].get(), "5");
    }

    #[test]
    fn map() {
        let json = json!({
            "a": 1,
            "b": {
                "c": [2, 3, 4],
                "d": [5]
            },
            "e": 6,
        })
        .to_string();

        let fields: HashMap<String, &RawValue> = extract_json_path(
            &json,
            hlist![
                "b",
                MultiMap(vec![
                    ("one".to_string(), hlist!["c", 1_usize]),
                    ("two".to_string(), hlist!["d", 0])
                ])
            ],
        )
        .unwrap();

        assert_eq!(fields.len(), 2);
        assert_eq!(fields["one"].get(), "3");
        assert_eq!(fields["two"].get(), "5");
    }
}
