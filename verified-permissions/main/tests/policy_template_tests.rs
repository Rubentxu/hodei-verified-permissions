//! Integration tests for Policy Template functionality

use hodei_infrastructure::SqliteRepository;

#[tokio::test]
async fn test_policy_template_crud() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();

    // Create policy store
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Create policy template
    let template_statement = r#"
        permit(
            principal == ?principal,
            action == Action::"view",
            resource == ?resource
        );
    "#;

    let template = repo
        .create_policy_template(
            &store.id,
            "share-template",
            template_statement,
            Some("Share template for documents"),
        )
        .await
        .unwrap();

    assert_eq!(template.template_id, "share-template");
    assert_eq!(template.policy_store_id, store.id);
    assert!(template.statement.contains("?principal"));
    assert!(template.statement.contains("?resource"));

    // Get template
    let retrieved = repo
        .get_policy_template(&store.id, "share-template")
        .await
        .unwrap();

    assert_eq!(retrieved.template_id, template.template_id);
    assert_eq!(retrieved.statement, template.statement);

    // List templates
    let templates = repo.list_policy_templates(&store.id).await.unwrap();
    assert_eq!(templates.len(), 1);
    assert_eq!(templates[0].template_id, "share-template");

    // Delete template
    repo.delete_policy_template(&store.id, "share-template")
        .await
        .unwrap();

    // Verify deletion
    let result = repo.get_policy_template(&store.id, "share-template").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_validation() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Template without placeholders should fail
    let invalid_statement = r#"
        permit(
            principal == User::"alice",
            action == Action::"view",
            resource == Document::"doc123"
        );
    "#;

    let result = repo
        .create_policy_template(&store.id, "invalid", invalid_statement, None)
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("placeholder"));
}

#[tokio::test]
async fn test_template_with_principal_only() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let statement = r#"
        permit(
            principal == ?principal,
            action == Action::"view",
            resource
        );
    "#;

    let template = repo
        .create_policy_template(&store.id, "principal-only", statement, None)
        .await
        .unwrap();

    assert_eq!(template.template_id, "principal-only");
    assert!(template.statement.contains("?principal"));
}

#[tokio::test]
async fn test_template_with_resource_only() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let statement = r#"
        permit(
            principal,
            action == Action::"view",
            resource == ?resource
        );
    "#;

    let template = repo
        .create_policy_template(&store.id, "resource-only", statement, None)
        .await
        .unwrap();

    assert_eq!(template.template_id, "resource-only");
    assert!(template.statement.contains("?resource"));
}

#[tokio::test]
async fn test_multiple_templates() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    // Create multiple templates
    repo.create_policy_template(
        &store.id,
        "template1",
        "permit(principal == ?principal, action, resource);",
        Some("Template 1"),
    )
    .await
    .unwrap();

    repo.create_policy_template(
        &store.id,
        "template2",
        "permit(principal, action, resource == ?resource);",
        Some("Template 2"),
    )
    .await
    .unwrap();

    repo.create_policy_template(
        &store.id,
        "template3",
        "permit(principal == ?principal, action, resource == ?resource);",
        Some("Template 3"),
    )
    .await
    .unwrap();

    // List all templates
    let templates = repo.list_policy_templates(&store.id).await.unwrap();
    assert_eq!(templates.len(), 3);

    let ids: Vec<&str> = templates.iter().map(|t| t.template_id.as_str()).collect();
    assert!(ids.contains(&"template1"));
    assert!(ids.contains(&"template2"));
    assert!(ids.contains(&"template3"));
}

#[tokio::test]
async fn test_template_not_found() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    let result = repo.get_policy_template(&store.id, "nonexistent").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_template_cascade_delete() {
    let repo = SqliteRepository::new(":memory:").await.unwrap();
    let store = repo
        .create_policy_store("Test Store".to_string(), Some("Test Store".to_string()), vec![], "test_user".to_string())
        .await
        .unwrap();

    repo.create_policy_template(
        &store.id,
        "template",
        "permit(principal == ?principal, action, resource);",
        None,
    )
    .await
    .unwrap();

    // Delete policy store (should cascade delete template)
    repo.delete_policy_store(&store.id).await.unwrap();

    // Verify template is also deleted
    let result = repo.get_policy_template(&store.id, "template").await;
    assert!(result.is_err());
}
