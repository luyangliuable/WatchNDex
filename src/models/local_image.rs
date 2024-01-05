use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use chrono::{ DateTime, Utc };
use serde::ser::Serializer;
use serde::de::{self, Deserializer};

mod date_format {
    use super::*;
    
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

#[derive(Serialize, Deserialize, Debug)]
pub struct LocalImage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub image_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub year: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub month: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(with = "date_format")]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(with = "date_format")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub date_last_modified: Option<DateTime<Utc>>,
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}
