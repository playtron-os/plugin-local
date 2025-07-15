use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Deserializer;

pub const ISO_FORMAT: &str = "%+";
pub const DATE_ONLY_FORMAT: &str = "%Y-%m-%dT%H:%M:%S";

pub fn deserialize_date<'de, D>(s: String) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let dt = NaiveDateTime::parse_from_str(&s, ISO_FORMAT).or_else(|_| {
        NaiveDateTime::parse_from_str(&s, DATE_ONLY_FORMAT).map_err(serde::de::Error::custom)
    })?;
    Ok(DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
}

pub mod optional_date_serializer {
    use chrono::{DateTime, Utc};
    use serde::{self, Deserialize, Deserializer, Serializer};

    use super::{deserialize_date, ISO_FORMAT};

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(_dt) => {
                let s = format!("{}", _dt.format(ISO_FORMAT));
                serializer.serialize_str(&s)
            }
            None => serializer.serialize_none(),
        }
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        match Option::<String>::deserialize(deserializer)? {
            Some(d) => Ok(Some(deserialize_date::<'de, D>(d)?)),
            None => Ok(None),
        }
    }
}
