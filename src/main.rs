mod models;
mod database;
mod repository;
mod errors;
mod enums;

use std::io;
use chrono::Utc;
use crate::models::traits::indexable::Indexable;
use regex::Regex;
use models::{local_image::LocalImage, post::Post};
use mongodb::{results::{ InsertOneResult, UpdateResult }, bson::{ doc, to_bson, oid::ObjectId }};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher, Result as NotifyResult};
use std::path::Path;
use database::db::{ DB, DbSingleton };
use enums::file_actions::FileActions;
use errors::errors::Errors;
use repository::{ local_image_repo::LocalImageRepo, post_repo::PostRepo,  repo::MongoRepo };
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    if let Err(e) = DbSingleton::init().await {
        eprintln!("Failed to initialize database: {:?}", e);
        std::process::exit(1); // Exit the program indicating a failure
    }

    let database = DB.get().expect("Database has not been initialized.");
    println!("Connected to database: {}", database.name());
    let path = env!("FOLDER_TO_WATCH");
    println!("Watching {}", path);

    let local_image_repo = LocalImageRepo(MongoRepo::<LocalImage>::init("LocalImage", &database).await);
    let post_repo = PostRepo(MongoRepo::<Post>::init("Post", &database).await);

    if let Err(e) = async_watch(path, local_image_repo, post_repo).await {
        println!("Error: {:?}", e);
    }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (mut tx, rx) = channel(1);

    let watcher = RecommendedWatcher::new(
        move |res| {
            futures::executor::block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}



async fn get_file_action(event: Event, re: &Regex) -> Result<FileActions, Errors> {

    if event.kind.is_modify() {
        for changed_path in event.paths {
            if re.is_match(&changed_path.to_string_lossy()) {
                continue;
            }

            match changed_path.file_stem().and_then(|os_str| os_str.to_str()) {
                Some(file_name_str) => {
                    println!("{:?}, {:?}, ismodify: {:?}", &event.kind, changed_path, &event.kind.is_modify());
                    return Ok(FileActions::FileUpdateAction {
                        changed_path: changed_path.to_owned(),
                        file_name: file_name_str.to_string(),
                    });
                },
                None => {
                    println!("Error: No file name");
                    return Err(Errors::MissingFileError);
                },
            }
        }
    } else if event.kind.is_create() {
        for changed_path in event.paths {
            if re.is_match(&changed_path.to_string_lossy()) {
                continue;
            }

            match changed_path.file_stem().and_then(|os_str| os_str.to_str()) {
                Some(file_name_str) => {
                    return Ok(FileActions::FileCreateAction {
                        changed_path: changed_path.to_owned(),
                        file_name: file_name_str.to_string(),
                    });
                },
                None => {
                    println!("Error: No file name");
                    return Err(Errors::MissingFileError);
                },
            }
        }
    }

    Ok(FileActions::None)
}


enum IndexableType {
    Post,
    LocalImage,
}

enum IndexableEnum {
    Post(Box<Post>),
    LocalImage(Box<LocalImage>)
}

fn create_indexable_object(changed_path: &Path, file_extension: &str, file_name: &str) -> IndexableEnum {
    let mut path_map = HashMap::new();
    path_map.insert("/Users/blackfish/personal-portfolio/psdasdasdosts/", IndexableType::Post);
    path_map.insert("/Users/blackfish/personal-portfolio/posts/", IndexableType::LocalImage);

    if let Some(changed_path_str) = changed_path.to_str() {
        println!("{}", changed_path_str);
        match path_map.iter()
            .find(|(key, _)| changed_path_str.starts_with(*key))
            .map(|(_, &ref value)| value) {
                Some(IndexableType::Post) => IndexableEnum::Post(Box::new(Post {
                    id: None,
                    heading:  "test".to_string(),
                    author:  "Luyang Liu".to_string(),
                    post_type:  file_extension.to_string(),
                    year: 2023,
                    month: 12,
                    file_name: file_name.to_string(),
                    description:  None,
                    date_created: Some(Utc::now()),
                    date_last_modified: Some(Utc::now()),
                    tags: None,
                    reading_time_minutes: None,
                    is_featured: None,
                    in_progress: None,
                    active: None,
                    image: None,
                    checksum: None,
                    body: None
                })),
                Some(IndexableType::LocalImage) => IndexableEnum::LocalImage(Box::new(LocalImage {
                    id: None,
                    year: None,
                    month: None,
                    date_created: Some(Utc::now()),
                    date_last_modified: Some(Utc::now()),
                    description: None,
                    source: None,
                    image_type: file_extension.to_string(),
                    file_name: file_name.to_string(),
                })),
                None => panic!("Path does not match any known type"),
            }
    } else {
        panic!("Failed to convert path to string");
    }
}


async fn async_watch<P: AsRef<Path>>(path: P, local_image_repo: LocalImageRepo, post_repo: PostRepo) -> NotifyResult<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    let ignore_patterns: Vec<String> = vec![".#.*".to_string()];

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    let re = Regex::new(&ignore_patterns[0]).unwrap();

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                let a = get_file_action(event, &re).await;
                println!("A!!!! {:?}", a);
                match a {
                    Ok(FileActions::FileUpdateAction { changed_path, file_name }) => {
                        // let res = update_file_index(&changed_path, &repository, &file_name, id).await;
                        // println!("{:?}", res);
                    },
                    Ok(FileActions::FileCreateAction { changed_path, file_name }) => {
                        let changed_object = create_indexable_object(&changed_path, &changed_path.extension().and_then(|os_str| os_str.to_str()).unwrap_or_else(|| "No file extension"), &file_name);

                        match changed_object {
                            IndexableEnum::Post(boxed_post) => {
                                let _ = boxed_post.index(&post_repo).await;
                            },
                            IndexableEnum::LocalImage(boxed_image) => {
                                println!("{:?}", boxed_image);
                                let _ = boxed_image.index(&local_image_repo).await;
                            },
                        }
                    },
                    Ok(FileActions::None) => {},
                    Err(e) => println!("Error processing file action: {:?}", e),
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}

async fn update_file_index(changed_path: &Path, repository: &LocalImageRepo, file_name: &str, id: ObjectId) -> Result<UpdateResult, io::Error> {
    let file_extension = match changed_path.extension().and_then(|os_str| os_str.to_str()) {
        Some(ext) => ext.to_string(),
        None => {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to extract file extension."));
        }
    };

    let new_local_image = LocalImage {
        id: None,
        year: None,
        month: None,
        date_created: None,
        date_last_modified: Some(Utc::now()),
        description: None,
        source: None,
        image_type: file_extension,
        file_name: file_name.to_string(),
    };


    let update_doc = match to_bson(&new_local_image) {
        Ok(mongodb::bson::Bson::Document(doc)) => doc,
        Ok(_other) => panic!(),
        Err(_) => panic!()
    };

    let update = doc! { "$set": update_doc };

    match repository.0.update_one(id, update, None).await {
        Ok(inserted_one) => Ok(inserted_one),
        Err(e) => {
            Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }
}
