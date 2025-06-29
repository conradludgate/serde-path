pub mod json;
pub mod json_ser;
mod list;
mod map;
pub mod map_select;
mod multi;
pub mod predicate;

pub use multi::{MultiMap, MultiVec};

use serde::de;

pub trait FilterChain<'de> {
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>;
}

#[derive(Clone, Copy, Debug)]
pub struct Const<T>(pub T);

impl<'de, T> FilterChain<'de> for Const<T>
where
    T: de::Deserializer<'de>,
{
    fn filter<D, S>(self, seed: S, _d: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        seed.deserialize(self.0).map_err(de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Final;

impl<'de> FilterChain<'de> for Final {
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        seed.deserialize(deserializer)
    }
}

impl<'de, F1, F2> FilterChain<'de> for (F1, F2)
where
    F1: FilterChain<'de>,
    F2: FilterChain<'de>,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        let (head, filter) = self;
        head.filter(Chain { filter, seed }, deserializer)
    }
}

#[derive(Debug, Clone)]
pub struct Iter<I>(pub I);

impl<'de, I> FilterChain<'de> for Iter<I>
where
    I: Iterator<Item: FilterChain<'de>>,
{
    fn filter<D, S>(mut self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        match self.0.next() {
            None => Final.filter(seed, deserializer),
            Some(head) => head.filter(Chain { filter: self, seed }, deserializer),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Chain<F, S> {
    pub filter: F,
    pub seed: S,
}

impl<'de, F, S> de::DeserializeSeed<'de> for Chain<F, S>
where
    F: FilterChain<'de>,
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let Self { filter, seed } = self;
        filter.filter(seed, deserializer)
    }
}

#[macro_export]
macro_rules! hlist {
    [$head:expr $(, $expr:expr)* $(,)?] => {
        ($head, $crate::hlist![$($expr,)*])
    };
    [] => { $crate::Final };
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use serde_json::{json, value::RawValue};

    use crate::FilterChain;

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
    fn it_works() {
        let json = json!({ "a": 1, "b": {"c": [2, 3, 4]}, "d": 5}).to_string();
        let field: &RawValue = extract_json_path(&json, hlist!["b", "c", 1]).unwrap();
        assert_eq!(field.get(), "3")
    }
}
