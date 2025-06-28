use serde::de;

use crate::FilterChain;

impl<'de, S: de::DeserializeSeed<'de>> FilterChain<'de, S> for usize {
    type Value = S::Value;

    fn filter<D>(self, seed: S, deserializer: D) -> Result<S::Value, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ListVisitor {
            head: self,
            next: seed,
        })
    }
}

struct ListVisitor<S> {
    head: usize,
    next: S,
}

impl<'de, S> de::Visitor<'de> for ListVisitor<S>
where
    S: de::DeserializeSeed<'de>,
{
    type Value = S::Value;
    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "a sequence containing element {:?}", self.head)
    }

    #[inline]
    fn visit_seq<A: de::SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let Self { head, next } = self;
        let mut i = 0;
        let v = loop {
            if i == head {
                break seq.next_element_seed(next)?;
            }

            if seq.next_element::<de::IgnoredAny>()?.is_none() {
                break None;
            }
            i += 1;
        };

        let v = v.ok_or_else(|| de::Error::custom(format_args!("missing field `{head:?}`")))?;

        while seq.next_element::<de::IgnoredAny>()?.is_some() {}
        Ok(v)
    }
}
