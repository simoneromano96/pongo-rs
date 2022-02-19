use futures::stream::{TryStreamExt, StreamExt};
use mongodb::{bson::doc, options::FindOptions};
use mongodb::{options::ClientOptions, Client};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
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

    // List the names of the databases in that deployment.
    // for db_name in client.list_database_names(None, None).await? {
    //     println!("{}", db_name);
    // }
    Ok(client)
}

pub async fn temp() {
    let client = make_connection().await.unwrap();

    let db = client.database("books");

    let books = vec![
        Book {
            title: "The Grapes of Wrath".to_string(),
            author: "John Steinbeck".to_string(),
        },
        Book {
            title: "To Kill a Mockingbird".to_string(),
            author: "Harper Lee".to_string(),
        },
    ];

    // Get a handle to a collection of `Book`.
    let typed_collection = db.collection::<Book>("books");

    // Insert the books into "mydb.books" collection, no manual conversion to BSON necessary.
    typed_collection.insert_many(books, None).await.unwrap();

    // Query the books in the collection with a filter and an option.
    // let filter = doc! { "author": "George Orwell" };
    let find_options = FindOptions::builder().sort(doc! { "title": 1 }).build();
    let mut cursor = typed_collection.find(None, find_options).await.unwrap();

    // Iterate over the results of the cursor.
    while let Some(book) = cursor.try_next().await.unwrap() {
        println!("{book:#?}");
    }

    let books: Vec<Book> = typed_collection.find(None, None).await.unwrap().try_collect().await.unwrap();
    println!("{books:#?}");
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }

    #[tokio::test]
    async fn temp() {
        super::temp().await;
    }
}
