use std::collections::HashMap;

use serde::de;
use serde_json::value::RawValue;

use crate::FilterChain;

#[derive(Debug, Clone)]
pub struct MultiVec<F>(pub Vec<F>);

impl<'de, S, F> FilterChain<'de, S> for MultiVec<F>
where
    F: FilterChain<'de, S>,
    S: de::DeserializeSeed<'de> + Clone,
{
    type Value = Vec<F::Value>;

    fn filter<D>(self, seed: S, deserializer: D) -> Result<Vec<F::Value>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let raw = <&'de RawValue as de::Deserialize>::deserialize(deserializer)?;

        let mut list = Vec::with_capacity(self.0.len());
        for filter in self.0 {
            let value = filter
                .filter(seed.clone(), raw)
                .map_err(de::Error::custom)?;
            list.push(value);
        }
        Ok(list)
    }
}

#[derive(Debug, Clone)]
pub struct MultiMap<F>(pub Vec<(String, F)>);

impl<'de, S, F> FilterChain<'de, S> for MultiMap<F>
where
    F: FilterChain<'de, S>,
    S: de::DeserializeSeed<'de> + Clone,
{
    type Value = HashMap<String, F::Value>;

    fn filter<D>(self, seed: S, deserializer: D) -> Result<HashMap<String, F::Value>, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let raw = <&'de RawValue as de::Deserialize>::deserialize(deserializer)?;

        let mut map = HashMap::with_capacity(self.0.len());
        for (key, filter) in self.0 {
            let value = filter
                .filter(seed.clone(), raw)
                .map_err(de::Error::custom)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, marker::PhantomData};

    use serde_json::{json, value::RawValue};

    use crate::{FilterChain, MultiVec, hlist, multi::MultiMap};

    fn extract_json_path<'de, F, T>(
        json: &'de str,
        filter: F,
    ) -> Result<F::Value, serde_json::Error>
    where
        T: serde::Deserialize<'de>,
        F: FilterChain<'de, PhantomData<T>>,
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
