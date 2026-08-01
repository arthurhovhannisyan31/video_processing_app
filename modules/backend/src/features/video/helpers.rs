use serde::{Deserialize, Deserializer};

pub fn deserialize_string_to_f32<'de, D>(
  deserializer: D,
) -> Result<f32, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<f32>().map_err(serde::de::Error::custom)
}

pub fn deserialize_string_to_i64<'de, D>(
  deserializer: D,
) -> Result<i64, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<i64>().map_err(serde::de::Error::custom)
}
