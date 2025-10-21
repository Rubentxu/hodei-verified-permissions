//! Procedural macros for Hodei Verified Permissions SDK
//!
//! This crate provides derive macros and attribute macros to simplify
//! Cedar authorization integration with Axum applications.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

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
/// - `action`: Cedar action name (required)
/// - `resource`: Cedar resource type (default: "Application")
/// - `description`: Action description for documentation (optional)
/// - `principal`: Principal type (default: "User")
#[proc_macro_attribute]
pub fn cedar_action(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    
    // NOTE: This is a simplified proof-of-concept implementation.
    // A full implementation would:
    // 1. Parse args to extract action, resource, principal, description
    // 2. Generate a const metadata struct
    // 3. Register the metadata in a global inventory for schema generation
    // 4. Potentially integrate with utoipa for OpenAPI generation
    //
    // For now, we just pass through the function unchanged.
    // The real power comes from using SimpleRestMapping at runtime.
    
    let expanded = quote! {
        #[doc = "Cedar-protected handler"]
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
