mod models;
mod database;
mod repository;

use std::io;
use chrono::Utc;
use regex::Regex;
use models::local_image::LocalImage;
use mongodb::{ 
	  bson::{Document, doc},
    results::InsertOneResult,
	  Client,
	  Collection 
};
use futures::{
    channel::mpsc::{channel, Receiver},
    SinkExt, StreamExt,
};
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher, Result as NotifyResult};
use std::path::Path;
use database::db::{ DB, DbSingleton };
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


async fn async_watch<P: AsRef<Path>>(path: P, repository: LocalImageRepo) -> NotifyResult<()> {
    let (mut watcher, mut rx) = async_watcher()?;
    let ignore_patterns: Vec<String> = vec![".#.*".to_string()];

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    let re = Regex::new(&ignore_patterns[0]).unwrap();

    while let Some(res) = rx.next().await {
        match res {
            Ok(event) => {
                if event.kind.is_modify() {
                    for changed_path in event.paths {
                        if re.is_match(&changed_path.to_string_lossy()) {
                            continue;
                        }

                        match changed_path.file_stem().and_then(|os_str| os_str.to_str()) {
                            Some(file_name_str) => {
                                match repository.get_documents_by_file_name(file_name_str).await {
                                    Ok(result) if result.len() > 0 => {
                                        println!("DB to update existing file {:?}", file_name_str)
                                    },
                                    Ok(_result) => {
                                        // Create Action
                                        index_new_file(&changed_path, &repository, &file_name_str).await;
                                    },
                                    Err(e) => {
                                        println!("Error {}", e);
                                    }
                                };
                            },
                            None => {
                                if changed_path.file_name().is_none() {
                                    println!("The path does not have a file name component.");
                                } else {
                                    println!("File name contains non-Unicode characters and cannot be processed.");
                                }
                            },
                        }
                    }
                }
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }

    Ok(())
}



async fn update_file_index(changed_path: &Path, repository: &LocalImageRepo, file_name: &str) -> Result<InsertOneResult, io::Error> {
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
        date_created: Some(Utc::now()),
        date_last_modified: None,
        description: None,
        source: None,
        image_type: file_extension,
        file_name: file_name.to_string(),
    };

    // Here you need to handle the error from repository creation.
    match repository.0.create(new_local_image).await {
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
        date_last_modified: None,
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

async fn connect_db() -> Result<(), mongodb::error::Error> {
    let uri = env!("MONGO_URI");

    let client = Client::with_uri_str(uri).await?;
    let database = client.database("rustdb");
    let image_collection: Collection<Document> = database.collection("images");

    let my_image = image_collection.find_one(doc! {}, None).await?;

    println!("Found a movie:\n{:#?}", my_image);

    Ok(())
}
