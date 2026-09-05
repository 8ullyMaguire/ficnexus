use std::fs;
use std::path::{Path, PathBuf};

use crate::cache::EType;
use crate::error::AppResult;

/// Compute the cache path for a given etype, url_id, and hash
///
/// Directory structure: <cache_root>/<etype>/<prefix1>/<prefix2>/<prefix3>/<url_id>/<hash><suffix>
/// where prefix chunks are 3 chars each from the url_id.
pub fn cache_path(cache_root: &Path, etype: &EType, url_id: &str, hash: &str) -> PathBuf {
    let mut path = cache_root.join(etype.as_str());

    // Split url_id into 3-char directory chunks (up to 9 chars = 3 levels deep)
    let chars: Vec<char> = url_id.chars().collect();
    for i in (0..chars.len()).step_by(3).take(3) {
        let chunk: String = chars.iter().skip(i).take(3).collect();
        if !chunk.is_empty() {
            path = path.join(chunk);
        }
    }

    // Full url_id directory
    path = path.join(url_id);

    // Actual file: hash + suffix
    path.join(format!("{}{}", hash, etype.suffix()))
}

/// Ensure the cache directory structure exists
pub fn ensure_cache_dir(path: &Path) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

/// Check if a file exists at the given path
pub fn cache_file_exists(path: &Path) -> bool {
    path.exists()
}

/// Remove old cache files for a given url_id and etype (stale hashes)
pub fn clear_stale_cache(
    cache_root: &Path,
    etype: &EType,
    url_id: &str,
    keep_hash: &str,
) -> AppResult<()> {
    let dir = cache_root.join(etype.as_str()).join(url_id);
    if dir.exists() {
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    if let Some(stem) = path.file_stem() {
                        if stem != keep_hash {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

/// Compute MD5 hash of a file
pub fn file_md5(path: &Path) -> AppResult<String> {
    use md5::{Digest, Md5};
    let data = fs::read(path)?;
    let hash = Md5::digest(&data);
    Ok(hex::encode(hash))
}

/// Move a file from source to destination, creating parent dirs
pub fn move_to_cache(source: &Path, dest: &Path) -> AppResult<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(source, dest)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::EType;
    use std::path::Path;

    #[test]
    fn test_cache_path_short_url_id() {
        let root = Path::new("/cache");
        let url_id = "abc";
        let hash = "hash123";
        let path = cache_path(root, &EType::Epub, url_id, hash);
        assert_eq!(path, Path::new("/cache/epub/abc/abc/hash123.epub"));
    }

    #[test]
    fn test_cache_path_medium_url_id() {
        let root = Path::new("/cache");
        let url_id = "abcdef";
        let hash = "h";
        let path = cache_path(root, &EType::Html, url_id, hash);
        assert_eq!(path, Path::new("/cache/html/abc/def/abcdef/h.zip"));
    }

    #[test]
    fn test_cache_path_long_url_id() {
        let root = Path::new("/cache");
        let url_id = "abcdefghijklm";
        let hash = "h1";
        let path = cache_path(root, &EType::Mobi, url_id, hash);
        assert_eq!(
            path,
            Path::new("/cache/mobi/abc/def/ghi/abcdefghijklm/h1.mobi")
        );
    }

    #[test]
    fn test_cache_path_different_etypes() {
        let root = Path::new("/cache");
        let url_id = "abc";
        let hash = "h";
        assert_eq!(
            cache_path(root, &EType::Epub, url_id, hash),
            Path::new("/cache/epub/abc/abc/h.epub")
        );
        assert_eq!(
            cache_path(root, &EType::Html, url_id, hash),
            Path::new("/cache/html/abc/abc/h.zip")
        );
        assert_eq!(
            cache_path(root, &EType::Mobi, url_id, hash),
            Path::new("/cache/mobi/abc/abc/h.mobi")
        );
        assert_eq!(
            cache_path(root, &EType::Pdf, url_id, hash),
            Path::new("/cache/pdf/abc/abc/h.pdf")
        );
    }

    #[test]
    fn test_file_md5_known_content() {
        let dir = std::env::temp_dir().join("fichub_test_disk_md5");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("test.txt");
        // MD5("hello world") = 5eb63bbbe01eeed093cb22bb8f5acdc3
        std::fs::write(&file_path, b"hello world").unwrap();
        let result = file_md5(&file_path).unwrap();
        assert_eq!(result, "5eb63bbbe01eeed093cb22bb8f5acdc3");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_md5_empty_file() {
        let dir = std::env::temp_dir().join("fichub_test_disk_empty");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file_path = dir.join("empty.txt");
        std::fs::write(&file_path, b"").unwrap();
        // MD5 of empty string = d41d8cd98f00b204e9800998ecf8427e
        let result = file_md5(&file_path).unwrap();
        assert_eq!(result, "d41d8cd98f00b204e9800998ecf8427e");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_file_md5_nonexistent_file() {
        let result = file_md5(Path::new("/nonexistent/path/file.txt"));
        assert!(result.is_err());
    }

    #[test]
    fn test_move_to_cache_creates_parent_and_moves() {
        let tmp = std::env::temp_dir().join("fichub_test_disk_move");
        let _ = std::fs::remove_dir_all(&tmp);

        let src_dir = tmp.join("source");
        std::fs::create_dir_all(&src_dir).unwrap();
        let src_file = src_dir.join("data.txt");
        std::fs::write(&src_file, b"test content").unwrap();

        let dest = tmp.join("cache").join("sub").join("final.txt");
        move_to_cache(&src_file, &dest).unwrap();

        assert!(!src_file.exists());
        assert!(dest.exists());
        assert_eq!(std::fs::read_to_string(&dest).unwrap(), "test content");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_move_to_cache_nonexistent_source() {
        let tmp = std::env::temp_dir().join("fichub_test_disk_move_fail");
        let _ = std::fs::remove_dir_all(&tmp);

        let src = tmp.join("nonexistent.txt");
        let dest = tmp.join("dest.txt");
        let result = move_to_cache(&src, &dest);
        assert!(result.is_err());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_cache_file_exists() {
        let tmp = std::env::temp_dir().join("fichub_test_disk_exists");
        let _ = std::fs::remove_dir_all(&tmp);
        std::fs::create_dir_all(&tmp).unwrap();

        let file = tmp.join("test.txt");
        assert!(!cache_file_exists(&file));

        std::fs::write(&file, b"data").unwrap();
        assert!(cache_file_exists(&file));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_ensure_cache_dir_creates_parents() {
        let tmp = std::env::temp_dir().join("fichub_test_disk_ensure");
        let _ = std::fs::remove_dir_all(&tmp);

        let deep = tmp.join("a").join("b").join("c").join("file.txt");
        ensure_cache_dir(&deep).unwrap();
        assert!(deep.parent().unwrap().exists());

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
