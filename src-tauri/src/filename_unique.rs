//! Display filename disambiguation: trailing ` (n)` copy suffix and batch uniqueness.
//!
//! Used for on-disk saves (with `std::fs`) and ZIP entry names (in-memory set only).

use std::collections::HashSet;
use std::path::Path;

/// If `stem` ends with ` (digits)`, returns `(prefix, Some(digits))`. Otherwise `(stem, None)`.
/// Only a **numeric** parenthetical at the end matches (e.g. `report (draft)` does not).
pub(crate) fn parse_trailing_copy_index(stem: &str) -> (&str, Option<u32>) {
    let Some(open) = stem.rfind(" (") else {
        return (stem, None);
    };
    let after_open = &stem[open + 2..];
    let Some(close_idx) = after_open.rfind(')') else {
        return (stem, None);
    };
    if close_idx + 1 != after_open.len() {
        return (stem, None);
    }
    let num_part = &after_open[..close_idx];
    if num_part.is_empty() || !num_part.chars().all(|c| c.is_ascii_digit()) {
        return (stem, None);
    }
    let Ok(n) = num_part.parse::<u32>() else {
        return (stem, None);
    };
    let base = stem[..open].trim_end();
    (base, Some(n))
}

fn split_display_name(filename: &str) -> (&str, Option<&str>) {
    let Some(dot) = filename.rfind('.') else {
        return (filename, None);
    };
    let stem = &filename[..dot];
    if stem.is_empty() {
        return (filename, None);
    }
    let ext = &filename[dot + 1..];
    if ext.is_empty() {
        return (filename, None);
    }
    (stem, Some(ext))
}

fn format_with_copy_index(logical_base: &str, n: u32, ext: Option<&str>) -> String {
    match ext {
        Some(e) if !e.is_empty() => format!("{logical_base} ({n}).{e}"),
        _ => format!("{logical_base} ({n})"),
    }
}

fn validate_leaf_filename(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }
    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
        return Err("Filename cannot contain path separators".to_string());
    }
    if trimmed == "." || trimmed == ".." {
        return Err("Invalid filename".to_string());
    }
    Ok(())
}

fn is_taken(
    folder: Option<&Path>,
    name: &str,
    reserved: &HashSet<String>,
    check_disk: bool,
) -> bool {
    if reserved.contains(name) {
        return true;
    }
    if check_disk {
        if let Some(dir) = folder {
            if dir.join(name).exists() {
                return true;
            }
        }
    }
    false
}

/// Pick the first non-colliding name for `proposed`, updating `reserved`.
fn allocate_unique(
    folder: Option<&Path>,
    proposed: &str,
    reserved: &mut HashSet<String>,
    check_disk: bool,
) -> Result<String, String> {
    validate_leaf_filename(proposed)?;
    let proposed = proposed.trim();

    if !is_taken(folder, proposed, reserved, check_disk) {
        reserved.insert(proposed.to_string());
        return Ok(proposed.to_string());
    }

    let (stem, ext) = split_display_name(proposed);
    let (logical_base, parsed_k) = parse_trailing_copy_index(stem);
    if logical_base.is_empty() {
        return Err("Invalid filename stem".to_string());
    }

    let start_n = parsed_k.map(|k| k.saturating_add(1)).unwrap_or(1);

    for n in start_n..10_000 {
        let candidate = format_with_copy_index(logical_base, n, ext);
        validate_leaf_filename(&candidate)?;
        if !is_taken(folder, &candidate, reserved, check_disk) {
            reserved.insert(candidate.clone());
            return Ok(candidate);
        }
    }

    Err(format!(
        "Could not find a free name for {:?} after thousands of attempts",
        proposed
    ))
}

/// Resolve each desired filename so the batch is unique vs disk (when `folder` is `Some`) and vs earlier slots.
pub fn resolve_unique_names_for_disk(
    folder: &Path,
    desired: &[String],
) -> Result<Vec<String>, String> {
    let mut reserved = HashSet::new();
    let mut out = Vec::with_capacity(desired.len());
    for name in desired {
        out.push(allocate_unique(Some(folder), name, &mut reserved, true)?);
    }
    Ok(out)
}

/// ZIP entry names: unique within the archive using the same ` (n)` convention (no disk).
pub fn uniquify_zip_entry_names(desired: &[String]) -> Result<Vec<String>, String> {
    let mut reserved = HashSet::new();
    let mut out = Vec::with_capacity(desired.len());
    for name in desired {
        out.push(allocate_unique(None, name, &mut reserved, false)?);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parse_trailing_copy_index_basic() {
        assert_eq!(parse_trailing_copy_index("photo"), ("photo", None));
        assert_eq!(parse_trailing_copy_index("photo (1)"), ("photo", Some(1)));
        assert_eq!(parse_trailing_copy_index("photo (12)"), ("photo", Some(12)));
    }

    #[test]
    fn parse_trailing_copy_index_non_numeric_suffix_unchanged() {
        let (base, n) = parse_trailing_copy_index("report (draft)");
        assert_eq!(base, "report (draft)");
        assert!(n.is_none());
    }

    #[test]
    fn resolve_skips_disk_and_increments_from_stem() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("a.png"), b"x").unwrap();
        fs::write(dir.path().join("a (1).png"), b"x").unwrap();

        let got = resolve_unique_names_for_disk(dir.path(), &[String::from("a.png")]).unwrap();
        assert_eq!(got, vec!["a (2).png"]);
    }

    #[test]
    fn resolve_batch_two_same_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        fs::write(dir.path().join("a.png"), b"x").unwrap();

        let got = resolve_unique_names_for_disk(
            dir.path(),
            &[String::from("a.png"), String::from("a.png")],
        )
        .unwrap();
        assert_eq!(got, vec!["a (1).png", "a (2).png"]);
    }

    #[test]
    fn uniquify_zip_two_same_names() {
        let got =
            uniquify_zip_entry_names(&[String::from("x.webp"), String::from("x.webp")]).unwrap();
        assert_eq!(got, vec!["x.webp", "x (1).webp"]);
    }
}
