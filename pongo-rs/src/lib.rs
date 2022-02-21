// use futures::stream::{TryStreamExt, StreamExt};
// use mongodb::{bson::doc, options::FindOptions};
// use mongodb::{options::ClientOptions, Client};
// use serde::{Deserialize, Serialize};

// #[derive(Debug, Serialize, Deserialize)]
// struct Book {
//     title: String,
//     author: String,
// }
use async_trait::async_trait;
use mongodb::{
    bson::{oid::ObjectId, Document},
    results::InsertOneResult,
    Collection, Cursor, Database,
};
use serde::{de::DeserializeOwned, Serialize};

pub type MongoError = mongodb::error::Error;

#[async_trait]
pub trait Model
where
    Self: Serialize + DeserializeOwned + Send + Sync + Unpin,
{
    /// The name of the collection where this model's data is stored.
    const COLLECTION_NAME: &'static str;
    // type Error;
    /// Get the ID for this model instance.
    fn get_id(&self) -> Option<ObjectId>;

    /// Set the ID for this model.
    fn set_id(&mut self, id: ObjectId);

    /// Gets mongo collection
    fn get_collection(db: &Database) -> Collection<Self>;

    async fn insert_one(db: &Database, document: &Self) -> Result<InsertOneResult, MongoError>;

    async fn find_by_id(db: &Database, id: &ObjectId) -> Result<Option<Self>, MongoError>;

    async fn find<F>(db: &Database, filter: F) -> Result<Cursor<Self>, MongoError>
    where
        F: Into<Option<Document>> + Send;

    async fn save(&self, db: &Database) -> Result<(), MongoError>;
}

pub mod prelude {
    pub use super::{Model, MongoError};
    pub use async_trait::async_trait;
    pub use mongodb::{
        bson::{doc, oid::ObjectId, Document},
        options::ClientOptions,
        results::InsertOneResult,
        Client, Collection, Cursor, Database,
    };
}

// pub async fn temp() {
//     let client = make_connection().await.unwrap();

//     let db = client.database("books");

//     let books = vec![
//         Book {
//             title: "The Grapes of Wrath".to_string(),
//             author: "John Steinbeck".to_string(),
//         },
//         Book {
//             title: "To Kill a Mockingbird".to_string(),
//             author: "Harper Lee".to_string(),
//         },
//     ];

//     // Get a handle to a collection of `Book`.
//     let typed_collection = db.collection::<Book>("books");

//     // Insert the books into "mydb.books" collection, no manual conversion to BSON necessary.
//     typed_collection.insert_many(books, None).await.unwrap();

//     // Query the books in the collection with a filter and an option.
//     // let filter = doc! { "author": "George Orwell" };
//     let find_options = FindOptions::builder().sort(doc! { "title": 1 }).build();
//     let mut cursor = typed_collection.find(None, find_options).await.unwrap();

//     // Iterate over the results of the cursor.
//     while let Some(book) = cursor.try_next().await.unwrap() {
//         println!("{book:#?}");
//     }

//     let books: Vec<Book> = typed_collection.find(None, None).await.unwrap().try_collect().await.unwrap();
//     println!("{books:#?}");
// }

// #[cfg(test)]
// mod tests {
//     #[tokio::test]
//     async fn it_works() {
//         let result = 2 + 2;
//         assert_eq!(result, 4);
//     }

//     #[tokio::test]
//     async fn temp() {
//         super::temp().await;
//     }
// }
