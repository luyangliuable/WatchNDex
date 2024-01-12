mod models;
mod database;
mod repository;
mod errors;
mod enums;

use std::io;
use chrono::Utc;
use regex::Regex;
use models::local_image::LocalImage;
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
use repository::{ local_image_repo::LocalImageRepo, repo::MongoRepo };

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

    let local_image_repo = LocalImageRepo(MongoRepo::<LocalImage>::init("images", &database).await);

    if let Err(e) = async_watch(path, local_image_repo).await {
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


async fn get_file_action(event: Event, repository: &LocalImageRepo, re: &Regex) -> Result<FileActions, Errors> {
    if event.kind.is_modify() {
        for changed_path in event.paths {
            if re.is_match(&changed_path.to_string_lossy()) {
                continue;
            }

            match changed_path.file_stem().and_then(|os_str| os_str.to_str()) {
                Some(file_name_str) => {
                    match repository.get_documents_by_file_name(file_name_str).await {
                        Ok(result) if result.len() > 0 => {
                            return Ok(FileActions::FileUpdateAction {
                                changed_path: changed_path.to_owned(),
                                file_name: file_name_str.to_string(),
                                id: result.get(0).and_then(|doc| doc.id).unwrap_or_else(|| panic!("No id found in the document"))
                                // id: result.get(0).and_then(|doc| doc.id).map_or_else(|| panic!(), |id| id)
                            });
                        },
                        Ok(_) => {
                            return Ok(FileActions::FileCreateAction {
                                changed_path: changed_path.to_owned(),
                                file_name: file_name_str.to_string(),
                            });
                        },
                        Err(e) => {
                            println!("Error {}", e);
                            return Err(Errors::FileListeningError);
                        }
                    }
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

async fn async_watch<P: AsRef<Path>>(path: P, repository: LocalImageRepo) -> NotifyResult<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    let ignore_patterns: Vec<String> = vec![".#.*".to_string()];

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    let re = Regex::new(&ignore_patterns[0]).unwrap();

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                match get_file_action(event, &repository, &re).await {
                    Ok(FileActions::FileUpdateAction { changed_path, file_name, id }) => {
                        let res = update_file_index(&changed_path, &repository, &file_name, id).await;
                        println!("{:?}", res);
                    },
                    Ok(FileActions::FileCreateAction { changed_path, file_name }) => {
                        let res = index_new_file(&changed_path, &repository, &file_name).await;
                        println!("{:?}", res);
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

async fn index_new_file(changed_path: &Path, repository: &LocalImageRepo, file_name: &str) -> Result<InsertOneResult, std::io::Error> {
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
        date_created: Some(chrono::offset::Utc::now()),
        date_last_modified: Some(chrono::offset::Utc::now()),
        description:  None,
        source: None,
        image_type:  file_extension.to_string(),
        file_name: file_name.to_string()
    };

    match repository.0.create(new_local_image).await {
        Ok(inserted_one) => Ok(inserted_one),
        Err(e) => {
            Err(io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
    }
}
