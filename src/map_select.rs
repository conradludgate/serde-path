use crate::{FilterChain, predicate::FilterPredicate};
use serde::de::{
    self, DeserializeSeed, MapAccess, SeqAccess,
    value::{MapAccessDeserializer, SeqAccessDeserializer},
};
use serde_json::value::RawValue;

pub struct MapSelect<F>(pub F);

impl<'de, F> FilterChain<'de> for MapSelect<F>
where
    F: FilterPredicate<'de> + Clone,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        deserializer.deserialize_any(MapSelectVisitor {
            filter: self.0,
            seed,
        })
    }
}

struct MapSelectVisitor<F, S> {
    filter: F,
    seed: S,
}

impl<'de, F, S> de::Visitor<'de> for MapSelectVisitor<F, S>
where
    S: de::DeserializeSeed<'de>,
    F: FilterPredicate<'de> + Clone,
{
    type Value = S::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a sequence or a map")
    }

    #[inline]
    fn visit_seq<A: de::SeqAccess<'de>>(self, seq: A) -> Result<Self::Value, A::Error> {
        self.seed
            .deserialize(SeqAccessDeserializer::new(MapSelectSeqAccess {
                filter: self.filter,
                seq,
            }))
            .map_err(de::Error::custom)
    }

    #[inline]
    fn visit_map<A: de::MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        self.seed
            .deserialize(MapAccessDeserializer::new(MapSelectMapAccess {
                filter: self.filter,
                value: None,
                map,
            }))
            .map_err(de::Error::custom)
    }
}

struct MapSelectSeqAccess<F, S> {
    filter: F,
    seq: S,
}

impl<'de, F, S> SeqAccess<'de> for MapSelectSeqAccess<F, S>
where
    S: SeqAccess<'de>,
    F: FilterPredicate<'de> + Clone,
{
    type Error = S::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        while let Some(val) = self.seq.next_element::<&'de RawValue>()? {
            if self.filter.clone().filter(val).map_err(de::Error::custom)? {
                return Ok(Some(seed.deserialize(val).map_err(de::Error::custom)?));
            }
        }
        Ok(None)
    }
}

struct MapSelectMapAccess<'de, F, M> {
    filter: F,
    value: Option<&'de RawValue>,
    map: M,
}

impl<'de, F, S> MapAccess<'de> for MapSelectMapAccess<'de, F, S>
where
    S: MapAccess<'de>,
    F: FilterPredicate<'de> + Clone,
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
        while let Some((key, val)) = self.map.next_entry::<&'de RawValue, &'de RawValue>()? {
            if self.filter.clone().filter(val).map_err(de::Error::custom)? {
                let key = kseed.deserialize(key).map_err(de::Error::custom)?;
                let val = vseed.deserialize(val).map_err(de::Error::custom)?;
                return Ok(Some((key, val)));
            }
        }
        Ok(None)
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        while let Some((key, val)) = self.map.next_entry::<&'de RawValue, &'de RawValue>()? {
            if self.filter.clone().filter(val).map_err(de::Error::custom)? {
                let key = seed.deserialize(key).map_err(de::Error::custom)?;
                return Ok(Some(key));
            }
        }
        Ok(None)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        seed.deserialize(self.value.unwrap())
            .map_err(de::Error::custom)
    }
}
