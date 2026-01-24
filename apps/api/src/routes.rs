use axum::{
    extract::Multipart,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use compression::compress_image_inproc;
use domain::CompressionOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CompressionResponse {
    pub original_size: usize,
    pub compressed_size: usize,
    pub savings_percent: f64,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BatchCompressionResponse {
    pub results: Vec<CompressionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// POST /api/compress
/// Compresses a single image file
pub async fn compress_image(mut multipart: Multipart) -> Result<impl IntoResponse, ApiError> {
    let mut file_data: Option<(Vec<u8>, String)> = None;
    let mut options = CompressionOptions::default();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name == "file" {
            // Extract filename before consuming field
            let filename = field.file_name()
                .unwrap_or("image")
                .to_string();
            
            // Extract extension
            let ext = filename
                .split('.')
                .last()
                .unwrap_or("png")
                .to_lowercase();
            
            let data = field.bytes().await.map_err(|e| {
                ApiError::BadRequest(format!("Failed to read file data: {}", e))
            })?;
            
            file_data = Some((data.to_vec(), ext));
        } else if name == "png_quality" {
            if let Ok(value) = field.text().await {
                options.png_quality = value;
            }
        } else if name == "png_lossy" {
            if let Ok(value) = field.text().await {
                options.png_lossy = value.parse().unwrap_or(true);
            }
        } else if name == "oxipng" {
            if let Ok(value) = field.text().await {
                options.oxipng = value.parse().unwrap_or(false);
            }
        } else if name == "to_webp" {
            if let Ok(value) = field.text().await {
                options.to_webp = value.parse().unwrap_or(false);
            }
        } else if name == "to_avif" {
            if let Ok(value) = field.text().await {
                options.to_avif = value.parse().unwrap_or(false);
            }
        } else if name == "to_jpeg" {
            if let Ok(value) = field.text().await {
                options.to_jpeg = value.parse().unwrap_or(false);
            }
        } else if name == "to_png" {
            if let Ok(value) = field.text().await {
                options.to_png = value.parse().unwrap_or(false);
            }
        }
    }

    let (file_bytes, ext) = file_data.ok_or_else(|| {
        ApiError::BadRequest("Missing 'file' field in multipart form".to_string())
    })?;

    let original_size = file_bytes.len();
    
    // Compress the image
    let (compressed_bytes, mime_type) = compress_image_inproc(&file_bytes, &ext, &options)
        .map_err(|e| ApiError::InternalError(format!("Compression failed: {}", e)))?;

    let compressed_size = compressed_bytes.len();
    let savings_percent = if original_size > 0 {
        ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
    } else {
        0.0
    };

    Ok(Json(CompressionResponse {
        original_size,
        compressed_size,
        savings_percent,
        mime_type,
        data: compressed_bytes,
    }))
}

/// POST /api/compress/batch
/// Compresses multiple image files
pub async fn compress_batch(mut multipart: Multipart) -> Result<impl IntoResponse, ApiError> {
    let mut files: Vec<(Vec<u8>, String)> = Vec::new();
    let mut options = CompressionOptions::default();

    // Parse multipart form data
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        
        if name.starts_with("file") {
            // Extract filename before consuming field
            let filename = field.file_name()
                .unwrap_or("image")
                .to_string();
            
            // Extract extension
            let ext = filename
                .split('.')
                .last()
                .unwrap_or("png")
                .to_lowercase();
            
            let data = field.bytes().await.map_err(|e| {
                ApiError::BadRequest(format!("Failed to read file data: {}", e))
            })?;
            
            files.push((data.to_vec(), ext));
        } else if name == "png_quality" {
            if let Ok(value) = field.text().await {
                options.png_quality = value;
            }
        } else if name == "png_lossy" {
            if let Ok(value) = field.text().await {
                options.png_lossy = value.parse().unwrap_or(true);
            }
        } else if name == "oxipng" {
            if let Ok(value) = field.text().await {
                options.oxipng = value.parse().unwrap_or(false);
            }
        }
    }

    if files.is_empty() {
        return Err(ApiError::BadRequest("No files provided".to_string()));
    }

    let mut results = Vec::new();

    for (file_bytes, ext) in files {
        let original_size = file_bytes.len();
        
        match compress_image_inproc(&file_bytes, &ext, &options) {
            Ok((compressed_bytes, mime_type)) => {
                let compressed_size = compressed_bytes.len();
                let savings_percent = if original_size > 0 {
                    ((original_size - compressed_size) as f64 / original_size as f64) * 100.0
                } else {
                    0.0
                };

                results.push(CompressionResponse {
                    original_size,
                    compressed_size,
                    savings_percent,
                    mime_type,
                    data: compressed_bytes,
                });
            }
            Err(e) => {
                // Continue with other files even if one fails
                results.push(CompressionResponse {
                    original_size,
                    compressed_size: 0,
                    savings_percent: 0.0,
                    mime_type: String::new(),
                    data: Vec::new(),
                });
                tracing::error!("Failed to compress file: {}", e);
            }
        }
    }

    Ok(Json(BatchCompressionResponse { results }))
}

/// API Error type
#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    InternalError(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            error: error_message,
        });

        (status, body).into_response()
    }
}
