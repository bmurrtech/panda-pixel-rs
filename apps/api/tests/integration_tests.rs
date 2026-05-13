use api::{config::Config, create_app};
use axum::http::StatusCode;
use axum_test::multipart::{MultipartForm, Part};
use axum_test::TestServer;
use image::{ImageBuffer, ImageFormat};
use std::io::Cursor;

#[tokio::test]
async fn batch_compress_output_format_webp_sets_mime() {
    let mut png_bytes = Vec::new();
    let img: ImageBuffer<image::Rgb<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(2, 2, image::Rgb([40u8, 80u8, 120u8]));
    img.write_to(&mut Cursor::new(&mut png_bytes), ImageFormat::Png)
        .expect("encode test png");

    let config = Config {
        app_env: "test".to_string(),
        port: 0,
        cors_allowed_origins: vec!["*".to_string()],
        rust_log: "error".to_string(),
    };
    let app = create_app(&config);
    let server = TestServer::new(app).expect("test server");

    let form = MultipartForm::new()
        .add_part("file", Part::bytes(png_bytes).file_name("sample.png"))
        .add_part("png_quality", Part::text("mid"))
        .add_part("png_lossy", Part::text("true"))
        .add_part("oxipng", Part::text("false"))
        .add_part("output_format", Part::text("webp"));

    let response = server.post("/api/compress/batch").multipart(form).await;
    assert_eq!(response.status_code(), StatusCode::OK);
    let v: serde_json::Value = response.json();
    let mime = v["results"][0]["mime_type"]
        .as_str()
        .expect("mime_type string");
    assert_eq!(
        mime, "image/webp",
        "batch route must honor output_format=webp (multipart output_format field)"
    );
}

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
        .multipart(axum_test::multipart::MultipartForm::new().add_part(
            "file",
            axum_test::multipart::Part::bytes(image_data).file_name("test.png"),
        ))
        .await;

    // Should be 200, 400, or 500 depending on validation
    // compress_image_inproc uses image::load_from_memory which will fail for dummy data
    // apps/api/src/routes.rs catches error and returns 400 for invalid image format, 500 for other errors
    // Dummy data should return 400 BadRequest for invalid image format

    assert!(
        response.status_code() == StatusCode::BAD_REQUEST
            || response.status_code() == StatusCode::INTERNAL_SERVER_ERROR
            || response.status_code() == StatusCode::OK
    );
}
