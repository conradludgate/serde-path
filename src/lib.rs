mod list;
mod map;

use serde::de;

pub trait FilterChain {
    fn filter<'de, S, D>(&self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        S: de::DeserializeSeed<'de>,
        D: de::Deserializer<'de>;
}

impl<F: FilterChain + ?Sized> FilterChain for &F {
    fn filter<'de, S, D>(&self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        S: de::DeserializeSeed<'de>,
        D: de::Deserializer<'de>,
    {
        F::filter(self, seed, deserializer)
    }
}

struct Final;

impl FilterChain for Final {
    fn filter<'de, S, D>(&self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        S: de::DeserializeSeed<'de>,
        D: de::Deserializer<'de>,
    {
        seed.deserialize(deserializer)
    }
}

impl<F1: FilterChain, F2: FilterChain> FilterChain for (F1, F2) {
    fn filter<'de, S, D>(&self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        S: de::DeserializeSeed<'de>,
        D: de::Deserializer<'de>,
    {
        self.0.filter(
            Chain {
                filter: &self.1,
                seed,
            },
            deserializer,
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Field<'a> {
    Index(usize),
    Key(&'a str),
}

impl FilterChain for Field<'_> {
    fn filter<'de, S, D>(&self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        S: de::DeserializeSeed<'de>,
        D: de::Deserializer<'de>,
    {
        match self {
            Field::Index(head) => head.filter(seed, deserializer),
            Field::Key(head) => head.filter(seed, deserializer),
        }
    }
}

impl<F: FilterChain> FilterChain for [F] {
    fn filter<'de, S, D>(&self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        S: de::DeserializeSeed<'de>,
        D: de::Deserializer<'de>,
    {
        match self {
            [] => Final.filter(seed, deserializer),
            [head, filter @ ..] => head.filter(Chain { filter, seed }, deserializer),
        }
    }
}

struct Chain<'a, F: ?Sized, S> {
    filter: &'a F,
    seed: S,
}

impl<'de, F, S> de::DeserializeSeed<'de> for Chain<'_, F, S>
where
    F: ?Sized + FilterChain,
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<S::Value, D::Error>
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
    use serde_json::{json, value::RawValue};

    use crate::FilterChain;

    fn extract_json_path<'de, T>(
        json: &'de str,
        filter: &impl FilterChain,
    ) -> Result<T, serde_json::Error>
    where
        T: serde::Deserialize<'de>,
    {
        filter.filter(
            std::marker::PhantomData::<T>,
            &mut serde_json::Deserializer::from_str(json),
        )
    }

    #[test]
    fn it_works() {
        let json = json!({ "a": 1, "b": {"c": [2, 3, 4]}, "d": 5}).to_string();
        let field: &RawValue = extract_json_path(&json, &hlist!["b", "c", 1]).unwrap();
        assert_eq!(field.get(), "3")
    }
}
