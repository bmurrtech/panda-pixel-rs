use std::path::Path;
use tokio::task;
use compression::{CompressionOptions, CompressionResult};

pub async fn compress_images_batch(
    file_paths: Vec<String>,
    options: CompressionOptions,
) -> Result<Vec<CompressionResult>, anyhow::Error> {
    let mut results = Vec::new();

    for file_path in file_paths {
        let options = options.clone();
        let result = task::spawn_blocking(move || {
            compress_single_image(&file_path, &options)
        }).await??;

        results.push(result);
    }

    Ok(results)
}

fn compress_single_image(
    input_path: &str,
    options: &CompressionOptions,
) -> Result<CompressionResult, anyhow::Error> {
    let input_path = Path::new(input_path);

    // Read input file
    let input_data = std::fs::read(input_path)?;

    // Determine output path
    let file_stem = input_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("output");

    let output_extension = match options.format {
        compression::OutputFormat::Auto => {
            // Keep original extension for Auto
            input_path.extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
        },
        compression::OutputFormat::Png => "png",
        compression::OutputFormat::Jpeg => "jpg",
        compression::OutputFormat::Webp => "webp",
        compression::OutputFormat::Avif => "avif",
        compression::OutputFormat::Tiff => "tiff",
        compression::OutputFormat::Bmp => "bmp",
        compression::OutputFormat::Ico => "ico",
    };

    let output_filename = format!("{}_compressed.{}", file_stem, output_extension);
    let output_path = input_path.with_file_name(output_filename);

    // Compress using the shared compression crate
    let compressed_data = compression::compress_image(&input_data, options)?;

    // Write output file
    std::fs::write(&output_path, &compressed_data)?;

    let original_size = input_data.len() as u64;
    let compressed_size = compressed_data.len() as u64;
    let savings = if original_size > 0 {
        ((compressed_size as f64 - original_size as f64) / original_size as f64 * 100.0).round()
            as i32
    } else {
        0
    };

    Ok(CompressionResult {
        original_path: input_path.to_string_lossy().to_string(),
        output_path: output_path.to_string_lossy().to_string(),
        original_size,
        compressed_size,
        savings_percent: savings,
        success: true,
        error_message: None,
    })
}