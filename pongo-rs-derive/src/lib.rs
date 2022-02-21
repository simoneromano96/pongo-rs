use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[derive(FromMeta, Debug)]
struct CollectionOptions {
    #[darling(default)]
    name: Option<String>,
}

/// Support parsing from a full derive input. Unlike FromMeta, this isn't
/// composable; each darling-dependent crate should have its own struct to handle
/// when its trait is derived.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(model), supports(struct_any))]
struct Model {
    // The struct ident.
    // ident: syn::Ident,
    #[darling(default)]
    collection_options: Option<CollectionOptions>,
}

#[proc_macro_derive(Model, attributes(model))]
pub fn model_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_model_derive_macro(&ast)
}

fn impl_model_derive_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let parsed: Model = FromDeriveInput::from_derive_input(&ast).unwrap();
    println!("{parsed:#?}");

    let collection_name = match parsed.collection_options {
        Some(collection_options) if collection_options.name.is_some() => {
            collection_options.name.unwrap()
        }
        _ => name.to_string(),
    };
    let collection_name = format_ident!("{}", collection_name);

    let gen = quote! {
      #[async_trait]
      impl Model for #name {
          const COLLECTION_NAME: &'static str = stringify!(#collection_name);
          
          fn set_id(&mut self, id: ObjectId) {
              self.id = Some(id);
          }

          fn get_id(&self) -> Option<ObjectId> {
              self.id
          }

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

          async fn find<F>(db: &Database, filter: F) -> Result<Cursor<Self>, MongoError>     
          where
            F: Into<Option<Document>> + Send {
                let typed_collection = Self::get_collection(db);
                typed_collection.find(filter, None).await
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
    };

    gen.into()
}

#[cfg(test)]
mod tests {
    use super::Model;
    use darling::FromDeriveInput;
    use syn::parse_str;

    #[test]
    fn test_specified_collection_name() {
        let input = r#"
        #[derive(Model)]
        #[model(collection_options(name = "test"))]
        struct Book {
            title: String,
            author: String,
        }
        "#;

        let parsed = parse_str(input).unwrap();
        let parsed: Model = FromDeriveInput::from_derive_input(&parsed).unwrap();

        assert_eq!(parsed.collection_options.unwrap().name.unwrap(), "test");
    }

    #[test]
    fn test_default_collection_name() {
        let input = r#"
        #[derive(Model)]
        struct Book {
            title: String,
            author: String,
        }
        "#;

        let parsed = parse_str(input).unwrap();
        let parsed: Model = FromDeriveInput::from_derive_input(&parsed).unwrap();

        assert!(parsed.collection_options.is_none());
    }
}
