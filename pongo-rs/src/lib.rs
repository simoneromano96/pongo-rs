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
    bson::{oid::ObjectId, Document, doc},
    results::InsertOneResult,
    Collection, Cursor, Database, options::FindOptions,
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
    /// Get the ID for this model instance.
    fn set_id(&mut self, id: ObjectId);

    /// Set the ID for this model.
    fn get_id(&self) -> Option<ObjectId>;

    /// Gets mongo collection
    fn get_collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(Self::COLLECTION_NAME)
    }

    async fn insert_one(
        db: &Database,
        document: &Self,
    ) -> Result<InsertOneResult, MongoError> {
        let typed_collection = Self::get_collection(db);
        typed_collection.insert_one(document, None).await
    }

    async fn find_by_id(
        db: &Database,
        id: &ObjectId,
    ) -> Result<Option<Self>, MongoError> {
        let typed_collection = Self::get_collection(db);
        let filter = doc! { "_id": id };
        typed_collection.find_one(filter, None).await
    }

    /// Find all instances of this model matching the given query.
    async fn find<F, O>(db: &Database, filter: F, options: O) -> Result<Cursor<Self>, MongoError>
    where
        F: Into<Option<Document>> + Send,
        O: Into<Option<FindOptions>> + Send,
    {
        let typed_collection = Self::get_collection(db);
        typed_collection.find(filter, options).await
    }


    async fn save(&self, db: &Database) -> Result<(), MongoError> {
        match self.get_id() {
            Some(_) => {
                let mut document = mongodb::bson::to_document(&self).unwrap();
                println!("{:#?}", document);
                if let Some(id) = document.remove("_id") {
                    let update_query = doc! { "$set": document };
                    let typed_collection = Self::get_collection(db);
                    typed_collection
                        .update_one(doc! { "_id": id }, update_query, None)
                        .await?;
                }
            }
            None => {
                Self::insert_one(db, self).await?;
            }
        };
        Ok(())
    }
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
