use darling::{FromDeriveInput, FromMeta};
use proc_macro::TokenStream;
use quote::{quote, format_ident};

#[derive(FromMeta, Default, Debug)]
struct CollectionOptions {
    #[darling(default)]
    name: Option<String>,
}

#[derive(FromDeriveInput, Default, Debug)]
#[darling(attributes(model))]
struct MyTrait {
    #[darling(default)]
    collection_options: Option<CollectionOptions>,
}

#[proc_macro_derive(MyTrait, attributes(model))]
pub fn myderive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_myderive_macro(&ast)
}

fn impl_myderive_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let parsed: MyTrait = FromDeriveInput::from_derive_input(&ast).unwrap();
    println!("{parsed:#?}");

    let collection_options = parsed.collection_options.unwrap_or_default();
    let collection_name = collection_options.name.unwrap_or_else(|| name.to_string());

    // fn get_collection(db: &Database) -> Collection<Self> {
    //   db.collection::<Self>(Self::get_collection_name())
    // }
    // stringify!();
    let c = format_ident!("{}", collection_name);

    let gen = quote! {
      #[async_trait]
      impl Model for #name {
          fn get_collection_name() -> &'static str {
            stringify!(#c)
            // format_ident!(#collection_name)
          }

          async fn find_by_id(db: &Database, id: &ObjectId) -> Self {
            let typed_collection = db.collection::<Self>(Self::get_collection_name());
            println!("{}", Self::get_collection_name());
            let filter = doc!{ "_id": id };
            typed_collection.find_one(filter, None).await.unwrap().unwrap()
          }
        }
    };
    gen.into()
}

#[cfg(test)]
mod tests {
    use super::MyTrait;
    use darling::FromDeriveInput;
    use syn::parse_str;

    #[test]
    fn test_specified_collection_name() {
        let input = r#"
        #[derive(MyTrait)]
        #[model(collection_options(name = "test"))]
        struct Book {
            title: String,
            author: String,
        }
        "#;

        let parsed = parse_str(input).unwrap();
        let parsed: MyTrait = FromDeriveInput::from_derive_input(&parsed).unwrap();

        assert_eq!(parsed.collection_options.unwrap().name.unwrap(), "test");
    }

    #[test]
    fn test_default_collection_name() {
        let input = r#"
        #[derive(MyTrait)]
        struct Book {
            title: String,
            author: String,
        }
        "#;

        let parsed = parse_str(input).unwrap();
        let parsed: MyTrait = FromDeriveInput::from_derive_input(&parsed).unwrap();

        assert!(parsed.collection_options.is_none());
    }
}
