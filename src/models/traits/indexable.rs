use mongodb::results::{ InsertOneResult, UpdateResult };
use std::path::Path;
use async_trait::async_trait;
use std::io;

#[async_trait]
pub trait Indexable<R>
{
    async fn index(&self, repository: &R) -> Result<InsertOneResult, io::Error>;

    async fn update<'a>(
        &'a self, 
        changed_path: &'a Path, 
        repository: &'a R, 
        file_name: &'a str
    ) -> Result<InsertOneResult, io::Error>;
}
