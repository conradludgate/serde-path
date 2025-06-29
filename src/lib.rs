pub mod json;
pub mod json_ser;
mod list;
mod map;
mod multi;

pub use multi::{MultiMap, MultiVec};

use serde::de;

pub trait FilterChain<'de, S: de::DeserializeSeed<'de>> {
    type Value;

    fn filter<D>(self, seed: S, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de>;
}

#[derive(Clone, Copy, Debug)]
struct Final;

impl<'de, S> FilterChain<'de, S> for Final
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;
    fn filter<D>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        seed.deserialize(deserializer)
    }
}

impl<'de, S, F1, F2> FilterChain<'de, S> for (F1, F2)
where
    S: de::DeserializeSeed<'de>,
    F1: FilterChain<'de, Chain<F2, S>>,
    F2: FilterChain<'de, S>,
{
    type Value = F1::Value;
    fn filter<D>(self, seed: S, deserializer: D) -> Result<F1::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let (head, filter) = self;
        head.filter(Chain { filter, seed }, deserializer)
    }
}

#[derive(Debug, Clone)]
pub struct Iter<I>(pub I);

impl<'de, S, I> FilterChain<'de, S> for Iter<I>
where
    S: de::DeserializeSeed<'de>,
    I: Iterator<Item: FilterChain<'de, Chain<Self, S>, Value = S::Value>>,
{
    type Value = S::Value;

    fn filter<D>(mut self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        match self.0.next() {
            None => Final.filter(seed, deserializer),
            Some(head) => head.filter(Chain { filter: self, seed }, deserializer),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Chain<F, S> {
    filter: F,
    seed: S,
}

impl<'de, F, S> de::DeserializeSeed<'de> for Chain<F, S>
where
    F: FilterChain<'de, S>,
    S: de::DeserializeSeed<'de>,
{
    type Value = F::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<F::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let Self { filter: path, seed } = self;
        path.filter(seed, deserializer)
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
    fn it_works() {
        let json = json!({ "a": 1, "b": {"c": [2, 3, 4]}, "d": 5}).to_string();
        let field: &RawValue = extract_json_path(&json, hlist!["b", "c", 1]).unwrap();
        assert_eq!(field.get(), "3")
    }
}
