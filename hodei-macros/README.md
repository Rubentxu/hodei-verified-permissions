# Hodei Macros

Procedural macros for Hodei Verified Permissions SDK to simplify Cedar authorization integration.

## Status: Proof of Concept

This crate provides a **proof-of-concept** implementation of procedural macros for Cedar authorization. The current implementation is intentionally simplified to demonstrate the approach.

## Design Philosophy

After evaluating different approaches for metaprogramming with Axum and Cedar, we've determined that **runtime mapping with SimpleRest is the optimal solution** for most use cases:

### Why Runtime Mapping is Preferred

1. **Flexibility**: Schema can be updated without recompiling
2. **Simplicity**: No complex macro magic or compile-time overhead
3. **Debugging**: Easier to debug runtime behavior vs compile-time generation
4. **Tooling**: Better IDE support and error messages
5. **Separation of Concerns**: OpenAPI spec remains the source of truth

### When Macros Make Sense

Macros are useful for:
- **Documentation**: Adding Cedar metadata to handler documentation
- **Type Safety**: Compile-time checks for action/resource types
- **Code Generation**: Generating boilerplate for common patterns
- **Integration**: Bridging with utoipa for OpenAPI generation

## Current Implementation

### `#[cedar_action]` Attribute Macro

```rust
#[cedar_action(
    action = "getDocument",
    resource = "Document",
    description = "Retrieve a document by ID"
)]
async fn get_document(Path(id): Path<String>) -> Json<Document> {
    // handler implementation
}
```

**Current behavior**: Adds documentation marker  
**Future behavior**: Could generate metadata for compile-time schema generation

### `#[derive(CedarEntity)]` Derive Macro

```rust
#[derive(CedarEntity)]
#[cedar(type = "Document")]
struct Document {
    #[cedar(id)]
    id: String,
    
    #[cedar(attribute)]
    title: String,
}
```

**Current behavior**: Placeholder  
**Future behavior**: Could implement `ToCedarEntity` trait automatically

## Recommended Approach

For production use, we recommend the **runtime mapping approach** implemented in Sprints 1 and 2:

```rust
// 1. Generate schema from OpenAPI
hodei-cli generate-schema --api-spec openapi.json --namespace MyApp

// 2. Load schema at runtime
let schema = std::fs::read_to_string("v4.cedarschema.json")?;
let layer = VerifiedPermissionsLayer::new(client, store_id, source_id)
    .with_simple_rest_mapping(&schema)?;

// 3. Apply to Axum router
let app = Router::new()
    .route("/documents/:id", get(get_document))
    .layer(layer);
```

This approach provides:
- ✅ Automatic action resolution
- ✅ Context extraction
- ✅ No compile-time overhead
- ✅ Easy schema updates
- ✅ Full type safety from Rust/Axum

## Future Enhancements

If there's demand for compile-time schema generation, we could implement:

### 1. Metadata Collection with `inventory`

```rust
use inventory;

#[cedar_action(action = "getDocument", resource = "Document")]
async fn get_document() { }

// Macro generates:
inventory::submit! {
    CedarActionMetadata {
        action: "getDocument",
        resource: "Document",
        // ...
    }
}

// build.rs collects all metadata and generates schema
```

### 2. Integration with `utoipa`

```rust
#[utoipa::path(
    get,
    path = "/documents/{id}",
    responses(
        (status = 200, description = "Success", body = Document)
    )
)]
#[cedar_action(action = "getDocument", resource = "Document")]
async fn get_document(Path(id): Path<String>) -> Json<Document> {
    // ...
}

// build.rs generates both OpenAPI and Cedar schema
```

### 3. Type-Safe Action Enums

```rust
// Generated from schema
#[derive(Debug, Clone, Copy)]
pub enum DocumentAction {
    GetDocument,
    CreateDocument,
    UpdateDocument,
    DeleteDocument,
}

impl DocumentAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GetDocument => "getDocument",
            // ...
        }
    }
}
```

## Implementation Complexity

A full macro implementation would require:

1. **Attribute Parsing**: Using `darling` or `syn` directly
2. **Metadata Storage**: Using `inventory` or `linkme` for global registration
3. **Build Script**: `build.rs` to collect metadata and generate schemas
4. **Proc-Macro Hygiene**: Careful handling of spans and identifiers
5. **Error Messages**: Good compile-time error reporting
6. **Documentation**: Extensive examples and edge case handling

**Estimated effort**: 2-3 weeks for a production-ready implementation

## Conclusion

The current runtime mapping approach provides **90% of the value with 10% of the complexity**. Macros are a nice-to-have for documentation and potential compile-time checks, but are not essential for a fully functional Cedar authorization system with Axum.

## References

- [Procedural Macros in Rust](https://doc.rust-lang.org/reference/procedural-macros.html)
- [inventory crate](https://docs.rs/inventory/)
- [utoipa](https://docs.rs/utoipa/)
- [darling](https://docs.rs/darling/)
