use std::marker::PhantomData;

use serde::{
    Deserializer,
    de::{self, DeserializeSeed, EnumAccess, MapAccess, SeqAccess, VariantAccess, Visitor},
};

#[derive(Clone, Copy)]
pub struct Unborrow<'a, T>(pub T, pub PhantomData<&'a ()>);

impl<'a, 'de, T: DeserializeSeed<'a>> DeserializeSeed<'de> for Unborrow<'a, T> {
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.deserialize(Unborrow(deserializer, PhantomData))
    }
}

impl<'a, 'de, T: Deserializer<'a>> Deserializer<'de> for Unborrow<'a, T> {
    type Error = T::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_any(Unborrow(visitor, PhantomData))
    }

    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_bool(Unborrow(visitor, PhantomData))
    }

    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i8(Unborrow(visitor, PhantomData))
    }

    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i16(Unborrow(visitor, PhantomData))
    }

    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i32(Unborrow(visitor, PhantomData))
    }

    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_i64(Unborrow(visitor, PhantomData))
    }

    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u8(Unborrow(visitor, PhantomData))
    }

    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u16(Unborrow(visitor, PhantomData))
    }

    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u32(Unborrow(visitor, PhantomData))
    }

    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_u64(Unborrow(visitor, PhantomData))
    }

    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_f32(Unborrow(visitor, PhantomData))
    }

    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_f64(Unborrow(visitor, PhantomData))
    }

    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_char(Unborrow(visitor, PhantomData))
    }

    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_str(Unborrow(visitor, PhantomData))
    }

    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_string(Unborrow(visitor, PhantomData))
    }

    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_bytes(Unborrow(visitor, PhantomData))
    }

    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_byte_buf(Unborrow(visitor, PhantomData))
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_option(Unborrow(visitor, PhantomData))
    }

    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_unit(Unborrow(visitor, PhantomData))
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_unit_struct(name, Unborrow(visitor, PhantomData))
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_newtype_struct(name, Unborrow(visitor, PhantomData))
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_seq(Unborrow(visitor, PhantomData))
    }

    fn deserialize_tuple<V: Visitor<'de>>(
        self,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_tuple(len, Unborrow(visitor, PhantomData))
    }

    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_tuple_struct(name, len, Unborrow(visitor, PhantomData))
    }

    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_map(Unborrow(visitor, PhantomData))
    }

    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_struct(name, fields, Unborrow(visitor, PhantomData))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_enum(name, variants, Unborrow(visitor, PhantomData))
    }

    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_identifier(Unborrow(visitor, PhantomData))
    }

    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0
            .deserialize_ignored_any(Unborrow(visitor, PhantomData))
    }
}

impl<'a, 'de, T: Visitor<'a>> Visitor<'de> for Unborrow<'a, T> {
    type Value = T::Value;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.expecting(formatter)
    }

    fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
        self.0.visit_bool(v)
    }

    fn visit_i8<E: de::Error>(self, v: i8) -> Result<Self::Value, E> {
        self.0.visit_i8(v)
    }

    fn visit_i16<E: de::Error>(self, v: i16) -> Result<Self::Value, E> {
        self.0.visit_i16(v)
    }

    fn visit_i32<E: de::Error>(self, v: i32) -> Result<Self::Value, E> {
        self.0.visit_i32(v)
    }

    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
        self.0.visit_i64(v)
    }

    fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E> {
        self.0.visit_i128(v)
    }

    fn visit_u8<E: de::Error>(self, v: u8) -> Result<Self::Value, E> {
        self.0.visit_u8(v)
    }

    fn visit_u16<E: de::Error>(self, v: u16) -> Result<Self::Value, E> {
        self.0.visit_u16(v)
    }

    fn visit_u32<E: de::Error>(self, v: u32) -> Result<Self::Value, E> {
        self.0.visit_u32(v)
    }

    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
        self.0.visit_u64(v)
    }

    fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E> {
        self.0.visit_u128(v)
    }

    fn visit_f32<E: de::Error>(self, v: f32) -> Result<Self::Value, E> {
        self.0.visit_f32(v)
    }

    fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
        self.0.visit_f64(v)
    }

    fn visit_char<E: de::Error>(self, v: char) -> Result<Self::Value, E> {
        self.0.visit_char(v)
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        self.0.visit_str(v)
    }

    fn visit_borrowed_str<E: de::Error>(self, v: &'de str) -> Result<Self::Value, E> {
        self.0.visit_str(v)
    }

    fn visit_string<E: de::Error>(self, v: String) -> Result<Self::Value, E> {
        self.0.visit_string(v)
    }

    fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
        self.0.visit_bytes(v)
    }

    fn visit_borrowed_bytes<E: de::Error>(self, v: &'de [u8]) -> Result<Self::Value, E> {
        self.0.visit_bytes(v)
    }

    fn visit_byte_buf<E: de::Error>(self, v: Vec<u8>) -> Result<Self::Value, E> {
        self.0.visit_byte_buf(v)
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        self.0.visit_none()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.visit_some(Unborrow(deserializer, PhantomData))
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        self.0.visit_unit()
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0
            .visit_newtype_struct(Unborrow(deserializer, PhantomData))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        self.0.visit_seq(Unborrow(seq, PhantomData))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: de::MapAccess<'de>,
    {
        self.0.visit_map(Unborrow(map, PhantomData))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: de::EnumAccess<'de>,
    {
        self.0.visit_enum(Unborrow(data, PhantomData))
    }
}

impl<'a, 'de, T: EnumAccess<'a>> EnumAccess<'de> for Unborrow<'a, T> {
    type Error = T::Error;
    type Variant = Unborrow<'a, T::Variant>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.0
            .variant_seed(Unborrow(seed, PhantomData))
            .map(|(val, var)| (val, Unborrow(var, PhantomData)))
    }
}

impl<'a, 'de, T: SeqAccess<'a>> SeqAccess<'de> for Unborrow<'a, T> {
    type Error = T::Error;

    fn next_element_seed<D>(&mut self, seed: D) -> Result<Option<D::Value>, Self::Error>
    where
        D: DeserializeSeed<'de>,
    {
        self.0.next_element_seed(Unborrow(seed, PhantomData))
    }
}

impl<'a, 'de, T: MapAccess<'a>> MapAccess<'de> for Unborrow<'a, T> {
    type Error = T::Error;

    fn next_entry_seed<K, V>(
        &mut self,
        kseed: K,
        vseed: V,
    ) -> Result<Option<(K::Value, V::Value)>, Self::Error>
    where
        K: DeserializeSeed<'de>,
        V: DeserializeSeed<'de>,
    {
        self.0
            .next_entry_seed(Unborrow(kseed, PhantomData), Unborrow(vseed, PhantomData))
    }

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        self.0.next_key_seed(Unborrow(seed, PhantomData))
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        self.0.next_value_seed(Unborrow(seed, PhantomData))
    }
}

impl<'a, 'de, T: VariantAccess<'a>> VariantAccess<'de> for Unborrow<'a, T> {
    type Error = T::Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        todo!()
    }

    fn newtype_variant_seed<D>(self, seed: D) -> Result<D::Value, Self::Error>
    where
        D: DeserializeSeed<'de>,
    {
        self.0.newtype_variant_seed(Unborrow(seed, PhantomData))
    }

    fn tuple_variant<V>(self, len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0.tuple_variant(len, Unborrow(visitor, PhantomData))
    }

    fn struct_variant<V>(
        self,
        fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.0
            .struct_variant(fields, Unborrow(visitor, PhantomData))
    }
}
