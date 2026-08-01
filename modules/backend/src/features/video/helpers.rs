use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize_string_to_i32<'de, D>(
  deserializer: D,
) -> Result<i32, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<i32>().map_err(serde::de::Error::custom)
}

pub fn deserialize_string_to_f32<'de, D>(
  deserializer: D,
) -> Result<f32, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<f32>().map_err(serde::de::Error::custom)
}

pub fn deserialize_string_to_f64<'de, D>(
  deserializer: D,
) -> Result<f64, D::Error>
where
  D: Deserializer<'de>,
{
  let s = String::deserialize(deserializer)?;
  s.parse::<f64>().map_err(serde::de::Error::custom)
}
