use std::collections::HashMap;

use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote, ToTokens};

/// The raw model used for deriving indices on models.
#[derive(Debug, FromMeta)]
struct RawIndexModel {
    #[darling(default)]
    #[darling(multiple)]
    #[darling(rename = "key")]
    keys: Vec<HashMap<String, i32>>,
}

impl From<&RawIndexModel> for mongodb::IndexModel {
    fn from(raw_index_model: &RawIndexModel) -> Self {
        let index_builder = mongodb::IndexModel::builder();
        let keys =
            raw_index_model
                .keys
                .iter()
                .fold(mongodb::bson::Document::new(), |mut acc, index| {
                    index.iter().for_each(|(key, order)| {
                        acc.extend([(key.clone(), order.into())]);
                    });
                    acc
                });
        let index_model = index_builder.keys(keys).options(None).build();
        index_model
    }
}

impl ToTokens for RawIndexModel {
    fn to_tokens(&self, input: &mut TokenStream2) {
        let keys = self.keys;
        // vec![]
        // HashMap::from_iter(vec![("key", "value")]);
        let token_stream = quote!(#(#keys),*);
        println!("{}", &token_stream);
        input.extend([token_stream]);
    }
}

#[derive(FromMeta, Debug)]
struct CollectionOptions {
    #[darling(default)]
    #[darling(map = "CollectionOptions::lower_case")]
    /// Collection name
    name: Option<String>,
}

impl CollectionOptions {
    fn lower_case(arg: Option<String>) -> Option<String> {
        if let Some(name) = arg {
            let new_name = name.chars().enumerate().fold(
                String::with_capacity(name.capacity()),
                |mut acc, (index, character)| {
                    if index == 0 {
                        acc.push(character.to_ascii_lowercase());
                    } else {
                        acc.push(character);
                    }
                    acc
                },
            );
            Some(new_name)
        } else {
            arg
        }
    }
}

/// Support parsing from a full derive input. Unlike FromMeta, this isn't
/// composable; each darling-dependent crate should have its own struct to handle
/// when its trait is derived.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(model), supports(struct_any))]
struct Model {
    #[darling(default)]
    /// All collection options
    collection_options: Option<CollectionOptions>,
    /// Collection indexes
    #[darling(default)]
    #[darling(multiple)]
    #[darling(rename = "index")]
    indexes: Vec<RawIndexModel>,
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
    let indexes = parsed.indexes;
    // let index_models: Vec<mongodb::IndexModel> =
    //     parsed.index.iter().map(|item| item.into()).collect();

    let gen = quote! {
      #[async_trait]
      impl Model for #name {
        const COLLECTION_NAME: &'static str = stringify!(#collection_name);

        /// Get the ID for this model instance.
        fn set_id(&mut self, id: ObjectId) {
          self.id = Some(id);
        }

        /// Set the ID for this model.
        fn get_id(&self) -> Option<ObjectId> {
          self.id
        }

        /// Get the vector of index models for this model.
        fn get_indexes() -> Vec<IndexModel> {
            vec![#(#indexes),*]
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
