use std::marker::PhantomData;

use serde::de::DeserializeSeed;

use crate::FilterChain;

#[derive(Debug, Clone, Copy)]
pub struct NotEq<F, T, S = PhantomData<T>> {
    filter: F,
    seed: S,
    value: T,
}

impl<F, T> NotEq<F, T> {
    pub fn new(filter: F, value: T) -> Self {
        Self {
            filter,
            seed: PhantomData,
            value,
        }
    }
}

impl<'de, F, T, S> FilterPredicate<'de> for NotEq<F, T, S>
where
    F: FilterChain<'de>,
    S: DeserializeSeed<'de, Value: PartialEq<T>>,
{
    fn filter<D>(self, deserializer: D) -> Result<bool, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let actual = self.filter.filter(self.seed, deserializer)?;
        Ok(actual != self.value)
    }
}

pub trait FilterPredicate<'de> {
    fn filter<D>(self, deserializer: D) -> Result<bool, D::Error>
    where
        D: serde::de::Deserializer<'de>;
}
