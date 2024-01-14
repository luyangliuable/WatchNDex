use crate::models::mongo_model::MongoModel;
use crate::repository::repo::MongoRepo;
use crate::models::post::Post;

impl MongoModel for Post {}
pub struct PostRepo(pub MongoRepo<Post>);

impl PostRepo {
}
