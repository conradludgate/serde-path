use std::marker::PhantomData;

use crate::{
    FilterChain, TakeWrapper,
    borrow::Unborrow,
    raw::{RawDeserializeSeed, RawValue2, WithRawValue},
};
use serde::de::{
    self, DeserializeSeed, IgnoredAny, MapAccess, SeqAccess,
    value::{MapAccessDeserializer, SeqAccessDeserializer},
};

pub struct Map<F>(pub F);

impl<'de, F> FilterChain<'de> for Map<F>
where
    F: FilterChain<'de> + Clone,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        deserializer.deserialize_any(MapVisitor {
            filter: self.0,
            seed,
        })
    }
}

struct MapVisitor<F, S> {
    filter: F,
    seed: S,
}

impl<'de, F, S> de::Visitor<'de> for MapVisitor<F, S>
where
    S: de::DeserializeSeed<'de>,
    F: FilterChain<'de> + Clone,
{
    type Value = S::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a sequence or a map")
    }

    #[inline]
    fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        self.seed
            .deserialize(SeqAccessDeserializer::new(MapValuesSeqAccess {
                filter: self.filter,
                seq,
            }))
            .map_err(de::Error::custom)
    }

    #[inline]
    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        self.seed
            .deserialize(SeqAccessDeserializer::new(MapMapSeqAccess {
                filter: self.filter,
                map,
            }))
            .map_err(de::Error::custom)
    }
}

pub struct MapValues<F>(pub F);

impl<'de, F> FilterChain<'de> for MapValues<F>
where
    F: FilterChain<'de> + Clone,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        deserializer.deserialize_any(MapValuesVisitor {
            filter: self.0,
            seed,
        })
    }
}

struct MapValuesVisitor<F, S> {
    filter: F,
    seed: S,
}

impl<'de, F, S> de::Visitor<'de> for MapValuesVisitor<F, S>
where
    S: de::DeserializeSeed<'de>,
    F: FilterChain<'de> + Clone,
{
    type Value = S::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a sequence or a map")
    }

    #[inline]
    fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        self.seed
            .deserialize(SeqAccessDeserializer::new(MapValuesSeqAccess {
                filter: self.filter,
                seq,
            }))
            .map_err(de::Error::custom)
    }

    #[inline]
    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        self.seed
            .deserialize(MapAccessDeserializer::new(MapValuesMapAccess {
                filter: self.filter,
                value: None,
                map,
            }))
            .map_err(de::Error::custom)
    }
}

struct Filter<'a, F, S> {
    seed: &'a mut TakeWrapper<S>,
    filter: F,
}

impl<'de, F, S> RawDeserializeSeed<'de> for Filter<'_, F, S>
where
    F: FilterChain<'de>,
    S: DeserializeSeed<'de>,
{
    type Value = Option<S::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de> + Clone,
    {
        let res = self.filter.filter(&mut *self.seed, deserializer.clone());
        match res {
            Ok(val) => Ok(Some(val)),
            Err(_) if self.seed.0.is_some() => Ok(None),
            Err(err) => Err(de::Error::custom(err)),
        }
    }
}

struct MapValuesSeqAccess<F, S> {
    filter: F,
    seq: S,
}

impl<'de, F, S> SeqAccess<'de> for MapValuesSeqAccess<F, S>
where
    S: SeqAccess<'de>,
    F: FilterChain<'de> + Clone,
{
    type Error = S::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let mut seed = TakeWrapper(Some(seed));

        loop {
            let filter = Filter {
                filter: self.filter.clone(),
                seed: &mut seed,
            };

            match self.seq.next_element_seed(WithRawValue(filter))? {
                Some(Some(val)) => break Ok(Some(val)),
                Some(None) => continue,
                None => break Ok(None),
            }
        }
    }
}

struct MapMapSeqAccess<F, M> {
    filter: F,
    map: M,
}

impl<'de, F, M> SeqAccess<'de> for MapMapSeqAccess<F, M>
where
    M: MapAccess<'de>,
    F: FilterChain<'de> + Clone,
{
    type Error = M::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let mut seed = TakeWrapper(Some(seed));

        loop {
            let filter = Filter {
                filter: self.filter.clone(),
                seed: &mut seed,
            };

            match self
                .map
                .next_entry_seed(PhantomData::<IgnoredAny>, WithRawValue(filter))?
            {
                Some((_key, Some(val))) => break Ok(Some(val)),
                Some((_key, None)) => continue,
                None => break Ok(None),
            }
        }
    }
}

struct MapValuesMapAccess<'de, F, M> {
    filter: F,
    value: Option<RawValue2<'de>>,
    map: M,
}

impl<'de, F, S> MapAccess<'de> for MapValuesMapAccess<'de, F, S>
where
    S: MapAccess<'de>,
    F: FilterChain<'de> + Clone,
{
    type Error = S::Error;

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        let mut vseed = TakeWrapper(Some(vseed));
        loop {
            let vfilter = WithRawValue(Filter {
                filter: self.filter.clone(),
                seed: &mut vseed,
            });
            let kfilter = PhantomData::<RawValue2<'de>>;

            match self.map.next_entry_seed(kfilter, vfilter)? {
                Some((key, Some(val))) => {
                    let key = kseed
                        .deserialize(Unborrow(&*key, PhantomData))
                        .map_err(de::Error::custom)?;
                    break Ok(Some((key, val)));
                }
                Some((_key, None)) => continue,
                None => break Ok(None),
            }
        }
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let mut vseed = TakeWrapper(Some(PhantomData::<RawValue2<'de>>));
        loop {
            let vfilter = WithRawValue(Filter {
                filter: self.filter.clone(),
                seed: &mut vseed,
            });
            let kfilter = PhantomData::<RawValue2<'de>>;

            match self.map.next_entry_seed(kfilter, vfilter)? {
                Some((key, Some(val))) => {
                    let key = seed
                        .deserialize(Unborrow(&*key, PhantomData))
                        .map_err(de::Error::custom)?;

                    self.value = Some(val);
                    break Ok(Some(key));
                }
                Some((_key, None)) => continue,
                None => break Ok(None),
            }
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        match self.value.as_ref().unwrap() {
            RawValue2::Borrowed(val) => seed.deserialize(*val).map_err(de::Error::custom),
            RawValue2::Owned(val) => seed
                .deserialize(Unborrow(&**val, PhantomData))
                .map_err(de::Error::custom),
        }
    }
}
