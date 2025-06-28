use std::marker::PhantomData;

use serde::de;

use crate::FilterChain;

impl<'de, S: de::DeserializeSeed<'de>> FilterChain<'de, S> for &str {
    type Value = S::Value;

    fn filter<D>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_map(MapVisitor {
            head: self,
            next: seed,
        })
    }
}

struct MapVisitor<'a, S> {
    head: &'a str,
    next: S,
}

impl<'de, S> de::Visitor<'de> for MapVisitor<'_, S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a map containing element {:?}", self.head)
    }

    #[inline]
    fn visit_map<A: de::MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let Self { head, next } = self;
        let visitor = FieldVisitor(head);

        let v = loop {
            let Some(found) = map.next_key_seed(visitor)? else {
                break None;
            };

            if found {
                break Some(map.next_value_seed(next)?);
            }

            map.next_value::<de::IgnoredAny>()?;
        };

        let v = v.ok_or_else(|| de::Error::custom(format_args!("missing field `{head}`")))?;

        while let Some((found, _)) = map.next_entry_seed(visitor, PhantomData::<de::IgnoredAny>)? {
            if found {
                return Err(de::Error::custom(format_args!("duplicate field `{head}`")));
            }
        }

        Ok(v)
    }
}

#[derive(Clone, Copy)]
struct FieldVisitor<'a>(&'a str);

impl<'de> de::Visitor<'de> for FieldVisitor<'_> {
    type Value = bool;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Formatter::write_str(formatter, "field identifier")
    }

    #[inline]
    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.visit_bytes(v.as_bytes())
    }

    #[inline]
    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        Ok(v == self.0.as_bytes())
    }
}

impl<'de> de::DeserializeSeed<'de> for FieldVisitor<'_> {
    type Value = bool;

    #[inline]
    fn deserialize<D>(self, deserializer: D) -> Result<bool, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_identifier(self)
    }
}
