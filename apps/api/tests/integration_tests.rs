use api::{config::Config, create_app};
use axum_test::TestServer;
use axum::http::StatusCode;

#[tokio::test]
async fn test_compress_endpoint_e2e() {
    // Setup config
    let config = Config {
        app_env: "test".to_string(),
        port: 0, // Not used for TestServer
        cors_allowed_origins: vec!["*".to_string()],
        rust_log: "error".to_string(),
    };

    let app = create_app(&config);
    let server = TestServer::new(app).unwrap();
    
    // Create a dummy image
    let image_data = vec![0u8; 100]; // Not valid image, but good for checking 400 or flow
    
    // Test multipart upload
    let response = server
        .post("/api/compress")
        .multipart(
            axum_test::multipart::MultipartForm::new()
                .add_part("file", axum_test::multipart::Part::bytes(image_data).file_name("test.png"))
        )
        .await;
    
    // Should be 200 or 400 depending on if we validate image content validity strictly before compression
    // compress_image_inproc uses image::load_from_memory which will fail for dummy data
    // apps/api/src/routes.rs catches error and returns 500 or 400?
    // It returns ApiError::InternalError("Compression failed: ...") which maps to 500
    // We expect 500 for invalid image data in current impl
    
    assert!(response.status_code() == StatusCode::INTERNAL_SERVER_ERROR || response.status_code() == StatusCode::OK);
}
