use std::io::Error;
use mongodb::{
    bson::{ doc, oid::ObjectId, Document },
    results::InsertOneResult
};
use mongodb::{Database, Collection};
use futures::stream::StreamExt;
use mongodb::results::UpdateResult;
use mongodb::options::UpdateOptions;
use crate::models::mongo_model::MongoModel;

pub struct MongoRepo<T: MongoModel> {
    pub insert_col: Collection<T>,
    pub get_col: Collection<T>
}

impl<T: MongoModel> MongoRepo<T> {
    pub async fn init(collection_name: &str, db: &Database) -> Self {
        let get_col: Collection<T> = db.collection(collection_name);
        let insert_col: Collection<T> = db.collection(collection_name);

        MongoRepo { insert_col, get_col }
    }

    pub async fn create(&self, new_entity: T) -> Result<InsertOneResult, Error> {
        let result = self
            .insert_col
            .insert_one(new_entity, None)
            .await.ok()
            .expect("Error creating entity");

        Ok(result)
    }

    pub async fn update_one(&self, id: ObjectId, update: Document, options: Option<UpdateOptions>) -> Result<UpdateResult, Error> {
        let filter = doc! {"_id": id};

        let options = match options {
            Some(options) => options,
            None => UpdateOptions::builder().upsert(true).build()
        };

        match self.insert_col.update_one(filter, update, options).await {
            Ok(result) if result.modified_count > 0 => {
                Ok(result)
            }
            Ok(result) => {
                Ok(result)
            }
            Err(_e) => {
                Err(Error::new(std::io::ErrorKind::Other, "Token Update failed."))
            }
        }
    }

    pub async fn get(&self, id: ObjectId) -> Result<T, Error> {
        let filter = doc! { "_id": id };

        let result = self
            .get_col
            .find_one(filter, None)
            .await
            .ok()
            .expect("Error getting item");

        match result {
            Some(document) => {
                Ok(document)
            },
            None => Err(Error::new(std::io::ErrorKind::Other, "Document not found")),
        }
    }

    pub async fn find_by_criteria(&self, criteria: Document) -> Result<Vec<T>, mongodb::error::Error> {
        let mut cursor = self.get_col.find(criteria, None).await?;
        let mut results = Vec::new();

        while let Some(result) = cursor.next().await {
            results.push(result?);
        }

        Ok(results)
    }

    pub async fn get_all(&self) -> Result<Vec<T>, Error> {
        let mut results: Vec<T> = Vec::new();

        let mut cursor = self
            .get_col
            .find(None, None)
            .await
            .ok()
            .expect("Error getting item");

        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => {
                    results.push(document);
                },
                Err(e) => {
                    return Err(Error::new(std::io::ErrorKind::Other, e.to_string()));
                }
            }
        }

        Ok(results)
    }
}
