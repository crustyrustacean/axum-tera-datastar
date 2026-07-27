// tests/api/items.rs

use crate::helpers::spawn_app;

#[tokio::test]
async fn post_new_item_returns_append_patch_and_clears_signal() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let response = app
        .api_client
        .post(format!("{}/items", &app.address))
        .json(&serde_json::json!({"item": "fish"}))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let body = response.text().await.unwrap();
    assert!(body.contains("event: datastar-patch-elements"));
    assert!(body.contains("data: selector #item-list"));
    assert!(body.contains("data: mode append"));
    assert!(body.contains("data: elements <li>fish</li>"));
    assert!(body.contains("event: datastar-patch-signals"));
}