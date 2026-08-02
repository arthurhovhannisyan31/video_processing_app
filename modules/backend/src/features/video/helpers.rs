use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

pub fn deserialize_string_to_type<'de, D, T>(
  deserializer: D,
) -> Result<T, D::Error>
where
  D: Deserializer<'de>,
  T: FromStr,
  T::Err: Display,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<T>().map_err(serde::de::Error::custom)
}
