use std::cell::Cell;
use std::io;

use serde::de::{DeserializeSeed, Deserializer, Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::ser::{Error as SerError, Serialize, SerializeMap, SerializeSeq, Serializer};
use serde_json::ser::Formatter;

pub struct JsonSer<W, F>(pub serde_json::Serializer<W, F>);

impl<'de, W, F> DeserializeSeed<'de> for JsonSer<W, F>
where
    W: io::Write,
    F: Formatter,
{
    type Value = W;

    fn deserialize<D: Deserializer<'de>>(mut self, d: D) -> Result<W, D::Error> {
        SerWrapper(&mut self.0).deserialize(d)?;
        Ok(self.0.into_inner())
    }
}

pub struct SerWrapper<S>(pub S);

impl<'de, S: Serializer> DeserializeSeed<'de> for SerWrapper<S> {
    type Value = S::Ok;

    fn deserialize<D: Deserializer<'de>>(self, d: D) -> Result<S::Ok, D::Error> {
        d.deserialize_any(self)
    }
}

impl<'de, S: Serializer> Visitor<'de> for SerWrapper<S> {
    type Value = S::Ok;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("any simple value")
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<S::Ok, A::Error> {
        let ser = self.0.serialize_seq(None).map_err(DeError::custom)?;
        let mut ser = TakeWrapper(Some(SerSeqWrapper(ser)));
        while let Some(ser_) = seq.next_element_seed(&mut ser)? {
            ser = TakeWrapper(Some(ser_));
        }
        ser.0.take().unwrap().0.end().map_err(DeError::custom)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut seq: A) -> Result<S::Ok, A::Error> {
        let ser = self.0.serialize_map(None).map_err(DeError::custom)?;
        let mut ser = TakeWrapper(Some(SerMapKeyWrapper(ser)));
        while let Some(ser_) = seq.next_key_seed(&mut ser)? {
            ser = TakeWrapper(Some(seq.next_value_seed(ser_)?));
        }
        ser.0.take().unwrap().0.end().map_err(DeError::custom)
    }

    fn visit_str<E: DeError>(self, v: &str) -> Result<S::Ok, E> {
        self.0.serialize_str(v).map_err(DeError::custom)
    }

    fn visit_bytes<E: DeError>(self, v: &[u8]) -> Result<S::Ok, E> {
        self.0.serialize_bytes(v).map_err(DeError::custom)
    }

    fn visit_i128<E: DeError>(self, v: i128) -> Result<S::Ok, E> {
        self.0.serialize_i128(v).map_err(DeError::custom)
    }

    fn visit_i64<E: DeError>(self, v: i64) -> Result<S::Ok, E> {
        self.0.serialize_i64(v).map_err(DeError::custom)
    }

    fn visit_i32<E: DeError>(self, v: i32) -> Result<S::Ok, E> {
        self.0.serialize_i32(v).map_err(DeError::custom)
    }

    fn visit_i16<E: DeError>(self, v: i16) -> Result<S::Ok, E> {
        self.0.serialize_i16(v).map_err(DeError::custom)
    }

    fn visit_i8<E: DeError>(self, v: i8) -> Result<S::Ok, E> {
        self.0.serialize_i8(v).map_err(DeError::custom)
    }

    fn visit_u128<E: DeError>(self, v: u128) -> Result<S::Ok, E> {
        self.0.serialize_u128(v).map_err(DeError::custom)
    }

    fn visit_u64<E: DeError>(self, v: u64) -> Result<S::Ok, E> {
        self.0.serialize_u64(v).map_err(DeError::custom)
    }

    fn visit_u32<E: DeError>(self, v: u32) -> Result<S::Ok, E> {
        self.0.serialize_u32(v).map_err(DeError::custom)
    }

    fn visit_u16<E: DeError>(self, v: u16) -> Result<S::Ok, E> {
        self.0.serialize_u16(v).map_err(DeError::custom)
    }

    fn visit_u8<E: DeError>(self, v: u8) -> Result<S::Ok, E> {
        self.0.serialize_u8(v).map_err(DeError::custom)
    }

    fn visit_bool<E: DeError>(self, v: bool) -> Result<S::Ok, E> {
        self.0.serialize_bool(v).map_err(DeError::custom)
    }

    fn visit_f32<E: DeError>(self, v: f32) -> Result<S::Ok, E> {
        self.0.serialize_f32(v).map_err(DeError::custom)
    }

    fn visit_f64<E: DeError>(self, v: f64) -> Result<S::Ok, E> {
        self.0.serialize_f64(v).map_err(DeError::custom)
    }

    fn visit_char<E: DeError>(self, v: char) -> Result<S::Ok, E> {
        self.0.serialize_char(v).map_err(DeError::custom)
    }

    fn visit_none<E: DeError>(self) -> Result<S::Ok, E> {
        self.0.serialize_none().map_err(DeError::custom)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<S::Ok, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0
            .serialize_some(&DeWrapper(Cell::new(Some(deserializer))))
            .map_err(DeError::custom)
    }

    fn visit_unit<E: DeError>(self) -> Result<S::Ok, E> {
        self.0.serialize_unit().map_err(DeError::custom)
    }
}

struct TakeWrapper<S>(Option<S>);
impl<'de, S: DeserializeSeed<'de>> DeserializeSeed<'de> for &mut TakeWrapper<S> {
    type Value = S::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.take().unwrap().deserialize(deserializer)
    }
}

struct SerSeqWrapper<S>(S);

impl<'de, S: SerializeSeq> DeserializeSeed<'de> for SerSeqWrapper<S> {
    type Value = Self;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0
            .serialize_element(&DeWrapper(Cell::new(Some(deserializer))))
            .map_err(DeError::custom)?;
        Ok(self)
    }
}

struct SerMapKeyWrapper<S>(S);

impl<'de, S: SerializeMap> DeserializeSeed<'de> for SerMapKeyWrapper<S> {
    type Value = SerMapValueWrapper<S>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0
            .serialize_key(&DeWrapper(Cell::new(Some(deserializer))))
            .map_err(DeError::custom)?;
        Ok(SerMapValueWrapper(self.0))
    }
}

struct SerMapValueWrapper<S>(S);

impl<'de, S: SerializeMap> DeserializeSeed<'de> for SerMapValueWrapper<S> {
    type Value = SerMapKeyWrapper<S>;

    fn deserialize<D>(mut self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0
            .serialize_value(&DeWrapper(Cell::new(Some(deserializer))))
            .map_err(DeError::custom)?;
        Ok(SerMapKeyWrapper(self.0))
    }
}

struct DeWrapper<D>(Cell<Option<D>>);

impl<'de, D: Deserializer<'de>> Serialize for DeWrapper<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerWrapper(serializer)
            .deserialize(
                self.0
                    .take()
                    .expect("DeWrapper serialize must only be used once"),
            )
            .map_err(SerError::custom)
    }
}
