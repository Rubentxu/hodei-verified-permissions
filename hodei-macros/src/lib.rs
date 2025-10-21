//! Procedural macros for Hodei Verified Permissions SDK
//!
//! This crate provides derive macros and attribute macros to simplify
//! Cedar authorization integration with Axum applications.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};
use darling::FromMeta;

/// Attribute macro to mark a handler with Cedar action metadata
///
/// # Example
///
/// ```rust,ignore
/// #[cedar_action(
///     action = "getDocument",
///     resource = "Document",
///     description = "Retrieve a document by ID"
/// )]
/// async fn get_document(Path(id): Path<String>) -> Json<Document> {
///     // handler implementation
/// }
/// ```
///
/// # Attributes
///
/// - `action`: Cedar action name (optional, for documentation)
/// - `resource`: Cedar resource type (default: "Application")
/// - `description`: Action description for documentation (optional)
#[derive(Debug, FromMeta)]
struct CedarActionArgs {
    #[darling(default)]
    action: Option<String>,
    #[darling(default = "default_resource")]
    resource: String,
    #[darling(default)]
    description: Option<String>,
}

fn default_resource() -> String {
    "Application".to_string()
}

#[proc_macro_attribute]
pub fn cedar_action(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match darling::ast::NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(darling::Error::from(e).write_errors());
        }
    };
    
    let args = match CedarActionArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };
    
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // Generate documentation
    let doc = if let Some(action) = &args.action {
        format!("Cedar Action: {} on {}", action, args.resource)
    } else {
        "Cedar-protected handler".to_string()
    };
    
    let desc_doc = args.description.as_ref().map(|d| {
        quote! {
            #[doc = #d]
        }
    });
    
    let expanded = quote! {
        #[doc = #doc]
        #desc_doc
        #input_fn
    };
    
    TokenStream::from(expanded)
}


/// Derive macro to automatically implement Cedar entity traits
///
/// # Example
///
/// ```rust,ignore
/// #[derive(CedarEntity)]
/// #[cedar(type = "Document")]
/// struct Document {
///     #[cedar(id)]
///     id: String,
///     
///     #[cedar(attribute)]
///     title: String,
///     
///     #[cedar(attribute)]
///     owner: String,
/// }
/// ```
#[proc_macro_derive(CedarEntity, attributes(cedar))]
pub fn derive_cedar_entity(_input: TokenStream) -> TokenStream {
    // For now, just return empty - this would need full implementation
    // with darling for attribute parsing
    TokenStream::new()
}
