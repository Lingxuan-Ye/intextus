use super::InlineString;
use core::fmt;
use serde_core::de::{Deserialize, Deserializer, Error, Visitor};
use serde_core::ser::{Serialize, Serializer};

impl<const N: usize> Serialize for InlineString<N> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self)
    }
}

impl<'de, const N: usize> Deserialize<'de> for InlineString<N> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = InlineStringVisitor::<N>;
        deserializer.deserialize_str(visitor)
    }
}

#[derive(Debug)]
struct InlineStringVisitor<const N: usize>;

impl<'de, const N: usize> Visitor<'de> for InlineStringVisitor<N> {
    type Value = InlineString<N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("struct InlineString")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: Error,
    {
        InlineString::<N>::try_from(value).map_err(E::custom)
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        InlineString::<N>::from_utf8(value).map_err(E::custom)
    }
}
