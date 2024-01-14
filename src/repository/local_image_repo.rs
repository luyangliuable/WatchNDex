use crate::models::mongo_model::MongoModel;
use crate::repository::repo::MongoRepo;
use crate::models::local_image::LocalImage;
use mongodb::bson::doc;

impl MongoModel for LocalImage {}
pub struct LocalImageRepo(pub MongoRepo<LocalImage>);


impl LocalImageRepo {
    pub async fn get_documents_by_file_name(&self, file_name: &str) -> Result<Vec<LocalImage>, mongodb::error::Error> {
        let criteria = doc! {
            "file_name": file_name
        };
        self.0.find_by_criteria(criteria).await
    }
}
