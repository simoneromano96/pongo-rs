use async_trait::async_trait;
use mongodb::{bson::doc, options::ClientOptions, Client, Database};
use pongo_rs::*;
use pongo_rs_derive::*;
use serde::{Deserialize, Serialize};
use mongodb::{bson::oid::ObjectId};

#[derive(Debug, MyTrait, Serialize, Deserialize)]
#[model(collection_options(name = "books"))]
struct Book {
    title: String,
    author: String,
}

async fn make_connection() -> Result<Client, mongodb::error::Error> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://root:example@localhost:27017").await?;

    // Manually set an option.
    client_options.app_name = Some("My App".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    Ok(client)
}

#[tokio::main]
async fn main() {
    // println!("hello");
    let client = make_connection().await.unwrap();
    let db = client.database("books");

    let name = Book::get_collection_name();
    println!("{name}");

    let instance = Book {
        title: "The Grapes of Wrath".to_string(),
        author: "John Steinbeck".to_string(),
    };

    let typed_collection = db.collection::<Book>(name);
    let insert_result = typed_collection.insert_one(&instance, None).await.unwrap();

    println!("{:#?}", &insert_result.inserted_id);

    let id = insert_result.inserted_id;

    match id {
        mongodb::bson::Bson::ObjectId(id) => {
            let book = Book::find_by_id(&db, &id).await;
            println!("{book:#?}");
        },
        _ => {}
    }
}
