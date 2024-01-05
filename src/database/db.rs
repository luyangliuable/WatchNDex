use mongodb::{Client, Database};
use std::env;
use once_cell::sync::OnceCell;

pub static DB: OnceCell<Database> = OnceCell::new();
pub struct DbSingleton;

impl DbSingleton {
    pub async fn init() -> Result<(), mongodb::error::Error> {
        let uri = env!("MONGO_URI");
        let database_name = env!("DATABASE_NAME");
        
        let client = Client::with_uri_str(uri).await?;
        let database = client.database(database_name);
        
        DB.set(database).unwrap();
        Ok(())
    }
}
