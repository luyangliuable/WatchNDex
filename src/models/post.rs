use mongodb::bson::oid::ObjectId;
use async_trait::async_trait;
use std::path::Path;
use mongodb::results::{ InsertOneResult, UpdateResult };
use serde::{Serialize, Deserialize};
use chrono::{ DateTime, Utc };
use std::io;
use crate::models::{ util::date_format::date_format, traits::indexable::Indexable };
use crate::repository::post_repo::PostRepo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub heading:  String,
    pub author:  String,
    pub post_type:  String,
    pub year: i32,
    pub month: i32,
    pub file_name: String,
    pub description:  Option<String>,
    #[serde(with = "date_format", default)]
    pub date_created: Option<DateTime<Utc>>,
    #[serde(with = "date_format", default)]
    pub date_last_modified: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
    pub reading_time_minutes: Option<i32>,
    pub is_featured: Option<bool>,
    pub in_progress: Option<bool>,
    pub active: Option<bool>,
    pub image: Option<ObjectId>,
    pub checksum: Option<String>,
    pub body: Option<String>
}


#[async_trait]
impl Indexable<PostRepo> for Post {
    async fn index(&self, repository: &PostRepo) -> Result<InsertOneResult, io::Error> {
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
        repository: &'a PostRepo, 
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
