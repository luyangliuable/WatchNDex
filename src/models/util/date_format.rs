
pub mod date_format {
    use chrono::{ DateTime, Utc };
    use serde::ser::Serializer;
    use serde::Deserialize;
    use serde::de::{self, Deserializer};

    const FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.fZ";  // ISO 8601 format

    pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match date {
            Some(d) => serializer.serialize_some(&d.format(FORMAT).to_string()),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<String> = Option::deserialize(deserializer)?;
        match s {
            Some(str) => DateTime::parse_from_rfc3339(&str)
                .map_err(de::Error::custom)
                .map(|dt| Some(dt.with_timezone(&Utc))),
            None => Ok(None),
        }
    }
}
