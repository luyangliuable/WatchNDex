use mongodb::bson::oid::ObjectId;
use serde::{Serialize, Deserialize};
use chrono::{ DateTime, Utc };
use async_trait::async_trait;
use mongodb::results::{ InsertOneResult, UpdateResult };
use crate::repository::local_image_repo::LocalImageRepo;
use crate::models::{ util::date_format::date_format, traits::indexable::Indexable };
use serde::de::{self, Deserializer};
use std::path::Path;
use std::io;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LocalImage {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub image_type: String,
    pub year: Option<i32>,
    pub month: Option<i32>,
    #[serde(with = "date_format", default)]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(with = "date_format", default)]
    pub date_last_modified: Option<DateTime<Utc>>,
    pub file_name: String,
    pub description: Option<String>,
    pub source: Option<String>
}

#[async_trait]
impl Indexable<LocalImageRepo> for LocalImage {
    async fn index(&self, repository: &LocalImageRepo) -> Result<InsertOneResult, std::io::Error> {
        match repository.0.create(self.clone()).await {
            Ok(inserted_one) => Ok(inserted_one),
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
            }
        }
    }

    async fn update<'a>(
        &'a self, 
        changed_path: &'a Path, 
        repository: &'a LocalImageRepo, 
        file_name: &'a str
    ) -> Result<InsertOneResult, io::Error> {
        let file_extension = match changed_path.extension().and_then(|os_str| os_str.to_str()) {
            Some(ext) => ext.to_string(),
            None => {
                return Err(io::Error::new(io::ErrorKind::Other, "Failed to extract file extension."));
            }
        };

        match repository.0.create(self.clone()).await {
            Ok(inserted_one) => Ok(inserted_one),
            Err(e) => {
                Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
            }
        }
    }
}
