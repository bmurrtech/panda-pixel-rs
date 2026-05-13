use crate::state::{CompressionResult, FileInfo};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt;
use wasm_bindgen::JsCast;
use web_sys::{window, Request, RequestInit, RequestMode, Response};
use wasm_bindgen_futures::JsFuture;

#[derive(Debug, Clone)]
pub enum BackendError {
    Cancelled,
    NotAvailable,
    CompressionFailed(String),
    SaveFailed(String),
    FileSelectionFailed(String),
    CollisionCheckFailed(String),
    Other(String),
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::Cancelled => write!(f, "Operation cancelled"),
            BackendError::NotAvailable => write!(f, "Backend not available"),
            BackendError::CompressionFailed(msg) => write!(f, "Compression failed: {}", msg),
            BackendError::SaveFailed(msg) => write!(f, "Save failed: {}", msg),
            BackendError::FileSelectionFailed(msg) => write!(f, "File selection failed: {}", msg),
            BackendError::CollisionCheckFailed(msg) => write!(f, "Collision check failed: {}", msg),
            BackendError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BackendError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionRequest {
    pub file_paths: Vec<String>,
    pub compression_level: String,
    pub output_format: String,
    pub oxipng: bool,
    pub png_lossy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFilesRequest {
    pub output_folder: String,
    pub files: Vec<FileSaveData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSaveData {
    pub filename: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveZipRequest {
    pub output_folder: String,
    pub zip_filename: String,
    pub files: Vec<FileSaveData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollisionCheckRequest {
    pub output_folder: String,
    pub filenames: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveUniqueFilenamesRequest {
    pub output_folder: String,
    pub filenames: Vec<String>,
}

// API Response types for HTTP backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiCompressionResponse {
    pub original_size: usize,
    pub compressed_size: usize,
    pub savings_percent: f64,
    pub mime_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiBatchCompressionResponse {
    pub results: Vec<ApiCompressionResponse>,
}

/// Detect image container from magic bytes (more reliable than trusting a mismatched `Content-Type` / JSON hint).
fn sniff_mime_from_bytes(data: &[u8]) -> Option<&'static str> {
    if data.len() >= 12 && &data[0..4] == b"RIFF" && &data[8..12] == b"WEBP" {
        return Some("image/webp");
    }
    if data.len() >= 3 && data[0] == 0xFF && data[1] == 0xD8 && data[2] == 0xFF {
        return Some("image/jpeg");
    }
    if data.len() >= 8 && data.get(0..4) == Some(b"\x89PNG") {
        return Some("image/png");
    }
    if data.len() >= 4 {
        let le = data.get(0..4) == Some(&[0x49, 0x49, 0x2A, 0x00]);
        let be = data.get(0..4) == Some(&[0x4D, 0x4D, 0x00, 0x2A]);
        if le || be {
            return Some("image/tiff");
        }
    }
    if data.len() >= 2 && data.get(0..2) == Some(b"BM") {
        return Some("image/bmp");
    }
    if data.len() > 12 && data.get(4..8) == Some(b"ftyp") {
        let cap = data.len().min(64);
        let head = &data[..cap];
        if head.windows(4).any(|w| w == b"avif" || w == b"avis") {
            return Some("image/avif");
        }
    }
    None
}

fn mime_from_bytes_or_hint(data: &[u8], server_mime: &str) -> String {
    if server_mime == "application/error" {
        return server_mime.to_string();
    }
    if let Some(sn) = sniff_mime_from_bytes(data) {
        return sn.to_string();
    }
    server_mime.to_string()
}

#[derive(Debug, Clone)]
pub struct BackendCapabilities {
    pub supports_native_dialogs: bool,
    pub supports_folder_picker: bool,
    pub supports_collision_check: bool,
    pub supports_file_manager: bool,
    pub supports_drag_drop: bool,
}

#[async_trait(?Send)]
pub trait AppBackend: fmt::Debug {
    fn capabilities(&self) -> BackendCapabilities;
    fn is_available(&self) -> bool;
    async fn select_files(&self) -> Result<Vec<FileInfo>, BackendError>;
    async fn handle_dropped_files(&self, file_paths: Vec<String>) -> Result<Vec<FileInfo>, BackendError>;
    async fn compress_batch(&self, request: CompressionRequest) -> Result<Vec<CompressionResult>, BackendError>;
    async fn select_output_folder(&self) -> Result<String, BackendError>;
    async fn save_files_to_folder(&self, request: SaveFilesRequest) -> Result<Vec<String>, BackendError>;
    async fn check_file_collisions(&self, request: CollisionCheckRequest) -> Result<Vec<String>, BackendError>;
    async fn resolve_unique_filenames(
        &self,
        request: ResolveUniqueFilenamesRequest,
    ) -> Result<Vec<String>, BackendError>;
    async fn save_files_as_zip(&self, request: SaveZipRequest) -> Result<String, BackendError>;
    async fn open_in_file_manager(&self, path: String) -> Result<(), BackendError>;
}

// Tauri Backend Implementation

#[derive(Debug, Clone)]
pub struct TauriBackend;

impl TauriBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TauriBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl AppBackend for TauriBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_native_dialogs: true,
            supports_folder_picker: true,
            supports_collision_check: true,
            supports_file_manager: true,
            supports_drag_drop: true,
        }
    }

    fn is_available(&self) -> bool {
        crate::tauri_helpers::is_tauri_available()
    }

    async fn select_files(&self) -> Result<Vec<FileInfo>, BackendError> {
        use wasm_bindgen::JsValue;
        
        let result = crate::tauri_helpers::invoke_tauri::<Vec<FileInfo>>("select_files", JsValue::NULL).await;
        
        match result {
            Ok(files) => Ok(files),
            Err(e) => {
                if e.contains("cancelled") || e.contains("Dialog cancelled") {
                    Err(BackendError::Cancelled)
                } else {
                    Err(BackendError::FileSelectionFailed(e))
                }
            }
        }
    }

    async fn handle_dropped_files(&self, file_paths: Vec<String>) -> Result<Vec<FileInfo>, BackendError> {
        use wasm_bindgen::JsValue;
        
        let args = js_sys::JSON::parse(
            &serde_json::to_string(&serde_json::json!({
                "filePaths": file_paths
            }))
            .unwrap_or_default(),
        )
        .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<Vec<FileInfo>>("handle_dropped_files", args).await;
        
        match result {
            Ok(files) => Ok(files),
            Err(e) => Err(BackendError::FileSelectionFailed(e)),
        }
    }

    async fn compress_batch(&self, request: CompressionRequest) -> Result<Vec<CompressionResult>, BackendError> {
        use wasm_bindgen::JsValue;
        
        let args_obj = serde_json::json!({
            "filePaths": request.file_paths,
            "compressionLevel": request.compression_level,
            "outputFormat": request.output_format,
            "oxipng": request.oxipng,
            "pngLossy": request.png_lossy,
        });

        let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
            .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<Vec<CompressionResult>>("compress_batch", args).await;
        
        match result {
            Ok(results) => Ok(results),
            Err(e) => Err(BackendError::CompressionFailed(e)),
        }
    }

    async fn select_output_folder(&self) -> Result<String, BackendError> {
        use wasm_bindgen::JsValue;
        
        let result = crate::tauri_helpers::invoke_tauri::<String>("select_output_folder", JsValue::NULL).await;
        
        match result {
            Ok(folder) => Ok(folder),
            Err(e) => {
                if e.contains("cancelled") || e.contains("No folder selected") {
                    Err(BackendError::Cancelled)
                } else {
                    Err(BackendError::Other(e))
                }
            }
        }
    }

    async fn save_files_to_folder(&self, request: SaveFilesRequest) -> Result<Vec<String>, BackendError> {
        use wasm_bindgen::JsValue;
        
        let files_json: Vec<serde_json::Value> = request.files.iter().map(|f| {
            serde_json::json!({
                "filename": f.filename,
                "data": f.data,
            })
        }).collect();

        let args_obj = serde_json::json!({
            "outputFolder": request.output_folder,
            "files": files_json,
        });

        let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
            .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<Vec<String>>("save_files_to_folder", args).await;
        
        match result {
            Ok(paths) => Ok(paths),
            Err(e) => Err(BackendError::SaveFailed(e)),
        }
    }

    async fn check_file_collisions(&self, request: CollisionCheckRequest) -> Result<Vec<String>, BackendError> {
        use wasm_bindgen::JsValue;
        
        let args_obj = serde_json::json!({
            "outputFolder": request.output_folder,
            "filenames": request.filenames,
        });

        let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
            .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<Vec<String>>("check_file_collisions", args).await;
        
        match result {
            Ok(collisions) => Ok(collisions),
            Err(e) => Err(BackendError::CollisionCheckFailed(e)),
        }
    }

    async fn resolve_unique_filenames(
        &self,
        request: ResolveUniqueFilenamesRequest,
    ) -> Result<Vec<String>, BackendError> {
        use wasm_bindgen::JsValue;

        let args_obj = serde_json::json!({
            "outputFolder": request.output_folder,
            "filenames": request.filenames,
        });

        let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
            .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<Vec<String>>("resolve_unique_filenames", args).await;

        match result {
            Ok(names) => Ok(names),
            Err(e) => Err(BackendError::CollisionCheckFailed(e)),
        }
    }

    async fn save_files_as_zip(&self, request: SaveZipRequest) -> Result<String, BackendError> {
        use wasm_bindgen::JsValue;
        
        let files_json: Vec<serde_json::Value> = request.files.iter().map(|f| {
            serde_json::json!({
                "filename": f.filename,
                "data": f.data,
            })
        }).collect();

        let args_obj = serde_json::json!({
            "outputFolder": request.output_folder,
            "zipFilename": request.zip_filename,
            "files": files_json,
        });

        let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
            .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<String>("save_files_as_zip", args).await;
        
        match result {
            Ok(path) => Ok(path),
            Err(e) => Err(BackendError::SaveFailed(e)),
        }
    }

    async fn open_in_file_manager(&self, path: String) -> Result<(), BackendError> {
        use wasm_bindgen::JsValue;
        
        let args_obj = serde_json::json!({
            "path": path,
        });

        let args = js_sys::JSON::parse(&serde_json::to_string(&args_obj).unwrap_or_default())
            .unwrap_or_else(|_| JsValue::NULL);

        let result = crate::tauri_helpers::invoke_tauri::<()>("open_in_file_manager", args).await;
        
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(BackendError::Other(e)),
        }
    }
}

// HTTP Backend Implementation

#[derive(Debug, Clone)]
pub struct HttpBackend {
    base_url: String,
}

impl HttpBackend {
    pub fn new() -> Self {
        let base_url = Self::get_api_base_url();
        Self { base_url }
    }

    fn get_api_base_url() -> String {
        if let Some(win) = window() {
            if let Ok(global_val) = js_sys::Reflect::get(&win, &wasm_bindgen::JsValue::from_str("__PANDA_API_URL__")) {
                if let Some(url) = global_val.as_string() {
                    if !url.is_empty() {
                        return url;
                    }
                }
            }
        }
        "http://localhost:3000".to_string()
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    async fn get(&self, endpoint: &str) -> Result<wasm_bindgen::JsValue, BackendError> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut opts = RequestInit::new();
        opts.method("GET");
        opts.mode(RequestMode::Cors);

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| BackendError::Other(format!("Failed to create request: {:?}", e)))?;

        let window = window().ok_or_else(|| BackendError::Other("No window object".to_string()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| BackendError::Other(format!("Fetch failed: {:?}", e)))?;

        let resp: Response = resp_value.dyn_into()
            .map_err(|e| BackendError::Other(format!("Invalid response: {:?}", e)))?;

        if !resp.ok() {
            return Err(BackendError::Other(format!("HTTP error: {}", resp.status())));
        }

        let json = JsFuture::from(resp.json().map_err(|e| BackendError::Other(format!("JSON error: {:?}", e)))?)
            .await
            .map_err(|e| BackendError::Other(format!("Failed to parse JSON: {:?}", e)))?;

        Ok(json)
    }

    async fn post_json(&self, endpoint: &str, body: &wasm_bindgen::JsValue) -> Result<wasm_bindgen::JsValue, BackendError> {
        let url = format!("{}{}", self.base_url, endpoint);
        
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(body));

        let request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| BackendError::Other(format!("Failed to create request: {:?}", e)))?;

        let window = window().ok_or_else(|| BackendError::Other("No window object".to_string()))?;
        let resp_value = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|e| BackendError::Other(format!("Fetch failed: {:?}", e)))?;

        let resp: Response = resp_value.dyn_into()
            .map_err(|e| BackendError::Other(format!("Invalid response: {:?}", e)))?;

        if !resp.ok() {
            return Err(BackendError::Other(format!("HTTP error: {}", resp.status())));
        }

        let json = JsFuture::from(resp.json().map_err(|e| BackendError::Other(format!("JSON error: {:?}", e)))?)
            .await
            .map_err(|e| BackendError::Other(format!("Failed to parse JSON: {:?}", e)))?;

        Ok(json)
    }
}

impl Default for HttpBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl AppBackend for HttpBackend {
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities {
            supports_native_dialogs: false,
            supports_folder_picker: false,
            supports_collision_check: false,
            supports_file_manager: false,
            supports_drag_drop: false,
        }
    }

    fn is_available(&self) -> bool {
        true
    }

    async fn select_files(&self) -> Result<Vec<FileInfo>, BackendError> {
        Err(BackendError::NotAvailable)
    }

    async fn handle_dropped_files(&self, _file_paths: Vec<String>) -> Result<Vec<FileInfo>, BackendError> {
        Err(BackendError::NotAvailable)
    }

    async fn compress_batch(&self, request: CompressionRequest) -> Result<Vec<CompressionResult>, BackendError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        use web_sys::{Request, RequestInit, RequestMode, Response};
        
        let window = window().ok_or_else(|| BackendError::Other("No window object".to_string()))?;

        // Get dropped files from global storage
        let dropped_files_result = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("__BROWSER_DROPPED_FILES"));

        let dropped_files = dropped_files_result
            .ok()
            .and_then(|val| val.dyn_into::<js_sys::Array>().ok())
            .unwrap_or_else(js_sys::Array::new);

        // Create multipart form data
        let form_data = web_sys::FormData::new()
            .map_err(|e| BackendError::CompressionFailed(format!("Failed to create form: {:?}", e)))?;
        
        form_data.append_with_str("png_quality", &request.compression_level)
            .map_err(|e| BackendError::CompressionFailed(format!("Failed to add field: {:?}", e)))?;
        form_data.append_with_str("png_lossy", &request.png_lossy.to_string())
            .map_err(|e| BackendError::CompressionFailed(format!("Failed to add field: {:?}", e)))?;
        form_data.append_with_str("oxipng", &request.oxipng.to_string())
            .map_err(|e| BackendError::CompressionFailed(format!("Failed to add field: {:?}", e)))?;
        form_data.append_with_str("output_format", &request.output_format)
            .map_err(|e| BackendError::CompressionFailed(format!("Failed to add output_format: {:?}", e)))?;

        let mut file_count = 0;
        for (i, path) in request.file_paths.iter().enumerate() {
            let file_obj = dropped_files.get(i as u32);
            let has_file = !file_obj.is_undefined() && !file_obj.is_null();

            if has_file {
                let file_result = js_sys::Reflect::get(&file_obj, &wasm_bindgen::JsValue::from_str("file"));

                if let Ok(file) = file_result {
                    let field_name = if i == 0 { "file".to_string() } else { format!("file{}", i) };
                    form_data.append_with_blob_and_filename(&field_name, &file.dyn_into().unwrap(), &path.replace("browser://", ""))
                        .map_err(|e| BackendError::CompressionFailed(format!("Failed to add file: {:?}", e)))?;
                    file_count += 1;
                }
            }
        }

        if file_count == 0 {
            return Err(BackendError::CompressionFailed("No files available to compress. Please select files using the file picker or drag & drop.".to_string()));
        }

        // Save file paths before creating HTTP request (which would shadow the request parameter)
        let file_paths = request.file_paths.clone();

        // Send request to API
        let url = format!("{}/api/compress/batch", self.base_url);
        let mut opts = RequestInit::new();
        opts.method("POST");
        opts.mode(RequestMode::Cors);
        opts.body(Some(&form_data));

        let http_request = Request::new_with_str_and_init(&url, &opts)
            .map_err(|e| BackendError::Other(format!("Failed to create request: {:?}", e)))?;

        let resp_value = JsFuture::from(window.fetch_with_request(&http_request))
            .await
            .map_err(|e| BackendError::Other(format!("Fetch failed: {:?}", e)))?;
        
        let resp: Response = resp_value.dyn_into()
            .map_err(|e| BackendError::Other(format!("Invalid response: {:?}", e)))?;

        if !resp.ok() {
            let status = resp.status();
            let text_promise = resp.text().unwrap_or_else(|_| {
                let arr = js_sys::Array::new();
                // `Array::join` returns `JsString`; newer `js_sys::Promise::resolve` is typed as
                // `Promise<JsValue>`, so coerce through `JsValue` for CI (no `src/Cargo.lock`).
                js_sys::Promise::resolve(&wasm_bindgen::JsValue::from(arr.join("")))
            });
            let text = JsFuture::from(text_promise)
                .await
                .ok()
                .and_then(|t| t.as_string())
                .unwrap_or_default();
            return Err(BackendError::CompressionFailed(format!("HTTP {}: {}", status, text)));
        }

        // Parse response
        let json = JsFuture::from(resp.json().map_err(|e| BackendError::Other(format!("JSON error: {:?}", e)))?)
            .await
            .map_err(|e| BackendError::Other(format!("Failed to parse JSON: {:?}", e)))?;

        let response_str = js_sys::JSON::stringify(&json)
            .map_err(|_| BackendError::Other("Failed to stringify response".to_string()))?
            .as_string()
            .ok_or_else(|| BackendError::Other("Invalid JSON response".to_string()))?;

        let batch_response: ApiBatchCompressionResponse = serde_json::from_str(&response_str)
            .map_err(|e| BackendError::Other(format!("Failed to parse response: {:?}", e)))?;

        // Convert API response to CompressionResult
        let req_fmt = request.output_format.clone();
        let results = batch_response
            .results
            .into_iter()
            .enumerate()
            .map(|(i, r)| {
                let mime = mime_from_bytes_or_hint(&r.data, &r.mime_type);
                CompressionResult {
                    original_path: file_paths.get(i).cloned().unwrap_or_else(|| format!("file{}", i)),
                    compressed_path: None,
                    original_size: r.original_size as u64,
                    compressed_size: r.compressed_size as u64,
                    savings_percent: r.savings_percent,
                    mime_type: mime,
                    data: r.data,
                    requested_output_format: Some(req_fmt.clone()),
                }
            })
            .collect();

        Ok(results)
    }

    async fn select_output_folder(&self) -> Result<String, BackendError> {
        // Browser mode doesn't have a folder picker, return a placeholder
        Ok("downloads".to_string())
    }

    async fn save_files_to_folder(&self, request: SaveFilesRequest) -> Result<Vec<String>, BackendError> {
        use wasm_bindgen::JsCast;
        
        let window = window().ok_or_else(|| BackendError::Other("No window object".to_string()))?;
        let document = window.document().ok_or_else(|| BackendError::Other("No document".to_string()))?;
        
        let mut downloaded_paths = Vec::new();
        
        for file_data in &request.files {
            let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
                &js_sys::Array::of1(&js_sys::Uint8Array::from(&file_data.data[..])),
                web_sys::BlobPropertyBag::new().type_("application/octet-stream"),
            ).map_err(|e| BackendError::SaveFailed(format!("Failed to create blob: {:?}", e)))?;
            
            let url = web_sys::Url::create_object_url_with_blob(&blob)
                .map_err(|e| BackendError::SaveFailed(format!("Failed to create URL: {:?}", e)))?;
            
            let anchor = document.create_element("a")
                .map_err(|e| BackendError::SaveFailed(format!("Failed to create anchor: {:?}", e)))?
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .map_err(|e| BackendError::SaveFailed(format!("Failed to cast anchor: {:?}", e)))?;
            
            anchor.set_href(&url);
            anchor.set_download(&file_data.filename);
            anchor.style().set_property("display", "none").ok();
            
            document.body().ok_or_else(|| BackendError::Other("No body".to_string()))?
                .append_child(&anchor)
                .map_err(|e| BackendError::SaveFailed(format!("Failed to append anchor: {:?}", e)))?;
            
            anchor.click();
            
            document.body().unwrap()
                .remove_child(&anchor)
                .map_err(|e| BackendError::SaveFailed(format!("Failed to remove anchor: {:?}", e)))?;
            
            web_sys::Url::revoke_object_url(&url);
            
            downloaded_paths.push(format!("downloads/{}", file_data.filename));

            crate::utils::product_log(&format!("💾 Downloaded: {}", file_data.filename));
        }
        
        Ok(downloaded_paths)
    }

    async fn check_file_collisions(&self, _request: CollisionCheckRequest) -> Result<Vec<String>, BackendError> {
        // Browser downloads handle collisions automatically
        Ok(Vec::new())
    }

    async fn resolve_unique_filenames(
        &self,
        request: ResolveUniqueFilenamesRequest,
    ) -> Result<Vec<String>, BackendError> {
        Ok(request.filenames)
    }

    async fn save_files_as_zip(&self, request: SaveZipRequest) -> Result<String, BackendError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen::JsValue;
        use wasm_bindgen_futures::JsFuture;

        let window = window().ok_or_else(|| BackendError::Other("No window object".to_string()))?;
        let document = window.document().ok_or_else(|| BackendError::Other("No document".to_string()))?;

        let zip_cb = js_sys::Reflect::get(&window, &JsValue::from_str("__pandaZipBuildAndDownload")).ok();
        if let Some(cb) = zip_cb {
            if let Ok(func) = cb.dyn_into::<js_sys::Function>() {
                let entries = js_sys::Array::new();
                for file_data in &request.files {
                    let o = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(
                        &o,
                        &JsValue::from_str("name"),
                        &JsValue::from_str(&file_data.filename),
                    );
                    let _ = js_sys::Reflect::set(
                        &o,
                        &JsValue::from_str("data"),
                        &JsValue::from(js_sys::Uint8Array::from(&file_data.data[..])),
                    );
                    entries.push(&o);
                }
                let zip_name = JsValue::from_str(&request.zip_filename);
                if let Ok(ret) = func.call2(&JsValue::NULL, &zip_name, &entries) {
                    let promise = js_sys::Promise::resolve(&ret);
                    match JsFuture::from(promise).await {
                        Ok(_) => {
                            crate::utils::product_log(&format!(
                                "💾 Downloaded ZIP bundle: {}",
                                request.zip_filename
                            ));
                            return Ok(format!("downloads/{}", request.zip_filename));
                        }
                        Err(e) => {
                            web_sys::console::warn_1(&JsValue::from_str(&format!(
                                "ZIP helper failed (falling back to sequential downloads): {:?}",
                                e
                            )));
                        }
                    }
                }
            }
        }

        for file_data in &request.files {
            let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
                &js_sys::Array::of1(&js_sys::Uint8Array::from(&file_data.data[..])),
                web_sys::BlobPropertyBag::new().type_("application/octet-stream"),
            )
            .map_err(|e| BackendError::SaveFailed(format!("Failed to create blob: {:?}", e)))?;

            let url = web_sys::Url::create_object_url_with_blob(&blob)
                .map_err(|e| BackendError::SaveFailed(format!("Failed to create URL: {:?}", e)))?;

            let anchor = document
                .create_element("a")
                .map_err(|e| BackendError::SaveFailed(format!("Failed to create anchor: {:?}", e)))?
                .dyn_into::<web_sys::HtmlAnchorElement>()
                .map_err(|e| BackendError::SaveFailed(format!("Failed to cast anchor: {:?}", e)))?;

            anchor.set_href(&url);
            anchor.set_download(&file_data.filename);
            let html_el: &web_sys::HtmlElement = anchor.unchecked_ref();
            let _ = html_el.style().set_property("display", "none");

            document
                .body()
                .ok_or_else(|| BackendError::Other("No body".to_string()))?
                .append_child(&anchor)
                .map_err(|e| BackendError::SaveFailed(format!("Failed to append anchor: {:?}", e)))?;

            anchor.click();

            document
                .body()
                .unwrap()
                .remove_child(&anchor)
                .map_err(|e| BackendError::SaveFailed(format!("Failed to remove anchor: {:?}", e)))?;

            let _ = web_sys::Url::revoke_object_url(&url);
        }

        Ok(format!("downloads/{}", request.zip_filename))
    }

    async fn open_in_file_manager(&self, _path: String) -> Result<(), BackendError> {
        // Browser can't open file manager
        Ok(())
    }
}

// Backend Provider

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BackendType {
    Tauri,
    Http,
}

#[derive(Debug, Clone)]
pub struct BackendProvider {
    backend_type: BackendType,
    tauri: TauriBackend,
    http: HttpBackend,
}

impl BackendProvider {
    pub fn new() -> Self {
        let tauri = TauriBackend::new();
        let http = HttpBackend::new();
        
        let backend_type = if tauri.is_available() {
            BackendType::Tauri
        } else {
            BackendType::Http
        };

        Self {
            backend_type,
            tauri,
            http,
        }
    }

    pub fn with_backend(backend_type: BackendType) -> Self {
        Self {
            backend_type,
            tauri: TauriBackend::new(),
            http: HttpBackend::new(),
        }
    }

    pub fn backend_type(&self) -> BackendType {
        self.backend_type
    }

    pub fn is_tauri(&self) -> bool {
        self.backend_type == BackendType::Tauri
    }

    pub fn is_http(&self) -> bool {
        self.backend_type == BackendType::Http
    }

    pub fn capabilities(&self) -> BackendCapabilities {
        match self.backend_type {
            BackendType::Tauri => self.tauri.capabilities(),
            BackendType::Http => self.http.capabilities(),
        }
    }

    fn current_backend(&self) -> &dyn AppBackend {
        match self.backend_type {
            BackendType::Tauri => &self.tauri,
            BackendType::Http => &self.http,
        }
    }
}

impl Default for BackendProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait(?Send)]
impl AppBackend for BackendProvider {
    fn capabilities(&self) -> BackendCapabilities {
        self.current_backend().capabilities()
    }

    fn is_available(&self) -> bool {
        self.current_backend().is_available()
    }

    async fn select_files(&self) -> Result<Vec<FileInfo>, BackendError> {
        self.current_backend().select_files().await
    }

    async fn handle_dropped_files(&self, file_paths: Vec<String>) -> Result<Vec<FileInfo>, BackendError> {
        self.current_backend().handle_dropped_files(file_paths).await
    }

    async fn compress_batch(&self, request: CompressionRequest) -> Result<Vec<CompressionResult>, BackendError> {
        self.current_backend().compress_batch(request).await
    }

    async fn select_output_folder(&self) -> Result<String, BackendError> {
        self.current_backend().select_output_folder().await
    }

    async fn save_files_to_folder(&self, request: SaveFilesRequest) -> Result<Vec<String>, BackendError> {
        self.current_backend().save_files_to_folder(request).await
    }

    async fn check_file_collisions(&self, request: CollisionCheckRequest) -> Result<Vec<String>, BackendError> {
        self.current_backend().check_file_collisions(request).await
    }

    async fn resolve_unique_filenames(
        &self,
        request: ResolveUniqueFilenamesRequest,
    ) -> Result<Vec<String>, BackendError> {
        self.current_backend().resolve_unique_filenames(request).await
    }

    async fn save_files_as_zip(&self, request: SaveZipRequest) -> Result<String, BackendError> {
        self.current_backend().save_files_as_zip(request).await
    }

    async fn open_in_file_manager(&self, path: String) -> Result<(), BackendError> {
        self.current_backend().open_in_file_manager(path).await
    }
}
