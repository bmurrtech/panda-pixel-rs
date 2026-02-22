use compression::compress_image_inproc;
use domain::CompressionOptions;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_compress_inproc_no_crash(data in prop::collection::vec(any::<u8>(), 0..10_000)) {
        let opts = CompressionOptions::default();
        // Just verify it doesn't panic. The result can be Ok or Err.
        let _ = compress_image_inproc(&data, "png", &opts);
    }

    #[test]
    fn test_compress_inproc_jpeg_no_crash(data in prop::collection::vec(any::<u8>(), 0..10_000)) {
        let opts = CompressionOptions::default();
        let _ = compress_image_inproc(&data, "jpg", &opts);
    }
}
