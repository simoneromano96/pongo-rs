use futures::TryStreamExt;
use pongo_rs::prelude::*;
use pongo_rs_derive::*;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Model, Serialize, Deserialize)]
#[model(collection_options(name = "books"))]
struct Book {
    id: Option<ObjectId>,
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

    let instance = Book {
        title: "The Grapes of Wrath".to_string(),
        author: "John Steinbeck".to_string(),
        ..Default::default()
    };

    // let typed_collection = db.collection::<Book>("books");
    // typed_collection.find(doc!{ "test": "test" }, None);

    let insert_result = Book::insert_one(&db, &instance).await.unwrap();

    println!("{:#?}", &insert_result.inserted_id);

    let id = insert_result.inserted_id;

    match id {
        mongodb::bson::Bson::ObjectId(id) => {
            let book = Book::find_by_id(&db, &id).await;
            println!("{book:#?}");
        }
        _ => {}
    }

    let books: Vec<Book> = Book::find(&db, None).await.unwrap().try_collect().await.unwrap();

    println!("{books:#?}");
}
