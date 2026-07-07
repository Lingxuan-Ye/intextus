use super::InlineDeque;
use core::fmt;
use core::marker::PhantomData;
use serde_core::de::{Deserialize, Deserializer, Error, SeqAccess, Visitor};
use serde_core::ser::{Serialize, Serializer};

impl<T, const N: usize> Serialize for InlineDeque<T, N>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_seq(self)
    }
}

impl<'de, T, const N: usize> Deserialize<'de> for InlineDeque<T, N>
where
    T: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = InlineDequeVisitor::<T, N>(PhantomData);
        deserializer.deserialize_seq(visitor)
    }
}

#[derive(Debug)]
struct InlineDequeVisitor<T, const N: usize>(PhantomData<T>);

impl<'de, T, const N: usize> Visitor<'de> for InlineDequeVisitor<T, N>
where
    T: Deserialize<'de>,
{
    type Value = InlineDeque<T, N>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> core::fmt::Result {
        formatter.write_str("struct InlineDeque")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut result = InlineDeque::<T, N>::new();
        while let Some(value) = seq.next_element()? {
            result.push_back(value).map_err(A::Error::custom)?;
        }
        Ok(result)
    }
}
