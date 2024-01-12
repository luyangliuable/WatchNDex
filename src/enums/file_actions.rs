use std::path::PathBuf;
use mongodb::bson::oid::ObjectId;

#[derive(Debug)]
pub enum FileActions {
    FileUpdateAction { changed_path: PathBuf, file_name: String, id: ObjectId },
    FileCreateAction { changed_path: PathBuf, file_name: String },
    None
}
