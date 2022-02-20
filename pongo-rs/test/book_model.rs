use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Book {
    title: String,
    author: String,
}
