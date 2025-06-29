use crate::{
    FilterChain,
    predicate::FilterPredicate,
    raw::{RawDeserializeSeed, WithRawValue},
};
use serde::de::{self, DeserializeSeed};

#[derive(Clone, Copy, Debug)]
pub struct Select<F>(pub F);

impl<'de, F> FilterChain<'de> for Select<F>
where
    F: FilterPredicate<'de>,
{
    fn filter<D, S>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: serde::de::Deserializer<'de>,
        S: de::DeserializeSeed<'de>,
    {
        WithRawValue(Select((self.0, seed))).deserialize(deserializer)
    }
}

impl<'de, F: FilterPredicate<'de>, S: de::DeserializeSeed<'de>> RawDeserializeSeed<'de>
    for Select<(F, S)>
{
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: de::Deserializer<'de> + Clone,
    {
        let (filter, seed) = self.0;
        let p = filter
            .filter(deserializer.clone())
            .map_err(de::Error::custom)?;

        if p {
            seed.deserialize(deserializer).map_err(de::Error::custom)
        } else {
            Err(de::Error::custom("filtered"))
        }
    }
}

// impl<'de, D: de::Deserializer<'de>, S: de::DeserializeSeed<'de>> de::Deserializer<'de>
//     for Select<(D, S)>
// {
//     type Error;

//     fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_unit_struct<V>(
//         self,
//         name: &'static str,
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_newtype_struct<V>(
//         self,
//         name: &'static str,
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_tuple<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_tuple_struct<V>(
//         self,
//         name: &'static str,
//         len: usize,
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_struct<V>(
//         self,
//         name: &'static str,
//         fields: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_enum<V>(
//         self,
//         name: &'static str,
//         variants: &'static [&'static str],
//         visitor: V,
//     ) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }

//     fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         todo!()
//     }
// }
