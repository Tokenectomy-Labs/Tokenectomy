use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;

pub fn get_cache_dir() -> Option<PathBuf> {
    if let Some(mut dir) = dirs::cache_dir() {
        dir.push("tokenectomy");
        dir.push("responses");
        fs::create_dir_all(&dir).ok()?;
        // VULN-08: Restrict cache directory permissions to owner only
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        Some(dir)
    } else {
        None
    }
}

pub fn hash_payload(payload: &str) -> String {
    hash_payload_with_source_context(payload, None)
}

/// Combines the raw error log with an optional hash of local workspace source context.
/// P19: Ensures cache invalidation if local code files change even if the error message is identical.
pub fn hash_payload_with_source_context(payload: &str, source_context_hash: Option<&str>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    if let Some(ctx_hash) = source_context_hash {
        hasher.update(b":source_context:");
        hasher.update(ctx_hash.as_bytes());
    }
    let result = hasher.finalize();
    let mut hash_str = String::new();
    for byte in result {
        hash_str.push_str(&format!("{:02x}", byte));
    }
    hash_str
}

pub fn get_cached_response(payload: &str) -> Option<String> {
    get_cached_sanitized_context(payload, None)
}

pub fn get_cached_sanitized_context(payload: &str, source_context_hash: Option<&str>) -> Option<String> {
    let dir = get_cache_dir()?;
    let hash = hash_payload_with_source_context(payload, source_context_hash);
    let cache_file = dir.join(format!("{}.txt", hash));

    if cache_file.exists() {
        if let Ok(metadata) = std::fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > CACHE_TTL_SECS {
                        let _ = std::fs::remove_file(&cache_file);
                        return None;
                    }
                }
            }
        }
        fs::read_to_string(cache_file).ok()
    } else {
        None
    }
}

const MAX_CACHE_ENTRIES: usize = 1000;
const CACHE_TTL_SECS: u64 = 86400; // 24 hours

pub fn save_cached_response(payload: &str, response: &str) {
    save_cached_sanitized_context(payload, None, response);
}

/// P19: Saves deterministic sanitized context (not LLM responses) with 24-hour TTL,
/// keyed by combined log payload and source context hash.
pub fn save_cached_sanitized_context(payload: &str, source_context_hash: Option<&str>, sanitized_context: &str) {
    if let Some(dir) = get_cache_dir() {
        let hash = hash_payload_with_source_context(payload, source_context_hash);
        let cache_file = dir.join(format!("{}.txt", hash));
        let tmp_file = dir.join(format!(
            "{}.tmp.{}.{}",
            hash,
            std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()
        ));

        // Atomic write: write to temp file then atomic rename
        if fs::write(&tmp_file, sanitized_context).is_ok() {
            let _ = fs::rename(&tmp_file, &cache_file);
        }

        // Bounded capacity housekeeping
        prune_cache_if_needed(&dir);
    }
}

/// Enforces cache TTL and maximum entry bounds to prevent disk exhaustion.
fn prune_cache_if_needed(dir: &std::path::Path) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut files_with_mtime = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("txt") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(elapsed) = mtime.elapsed() {
                        if elapsed.as_secs() > CACHE_TTL_SECS {
                            let _ = fs::remove_file(&path);
                            continue;
                        }
                    }
                    files_with_mtime.push((path, mtime));
                }
            }
        }
    }

    // If still exceeds maximum capacity, evict oldest entries
    if files_with_mtime.len() > MAX_CACHE_ENTRIES {
        files_with_mtime.sort_by_key(|(_, mtime)| *mtime);
        let to_remove = files_with_mtime.len() - MAX_CACHE_ENTRIES;
        for (path, _) in files_with_mtime.into_iter().take(to_remove) {
            let _ = fs::remove_file(path);
        }
    }
}

pub fn get_raw_dumps_dir() -> Option<PathBuf> {
    if let Some(mut dir) = dirs::cache_dir() {
        dir.push("tokenectomy");
        dir.push("raw_dumps");
        fs::create_dir_all(&dir).ok()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&dir, fs::Permissions::from_mode(0o700));
        }
        Some(dir)
    } else {
        None
    }
}

/// Stores the unpruned raw log payload in content-addressable storage indexed by its SHA-256 hash.
/// Returns the addressable URI identifier in standard `sha256:<digest>` format.
pub fn save_raw_dump(raw: &str) -> String {
    let hash = hash_payload(raw);
    let full_hash = format!("sha256:{}", hash);
    if let Some(dir) = get_raw_dumps_dir() {
        let dump_file = dir.join(format!("{}.log", hash));
        let needs_write = !dump_file.exists() || dump_file.metadata().map(|m| m.len() == 0).unwrap_or(true);
        if needs_write {
            let tmp_file = dir.join(format!(
                "{}.tmp.{}.{}",
                hash,
                std::process::id(),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos()
            ));
            if fs::write(&tmp_file, raw).is_ok() {
                let _ = fs::rename(&tmp_file, &dump_file);
            }
            prune_raw_dumps_if_needed(&dir);
        }
    }
    full_hash
}

/// Retrieves the raw dump from content-addressable storage using its SHA-256 digest or `sha256:<digest>` URI.
pub fn get_raw_dump(hash: &str) -> Option<String> {
    let clean_hash = hash.trim_start_matches("sha256:").trim();
    let dir = get_raw_dumps_dir()?;
    let dump_file = dir.join(format!("{}.log", clean_hash));
    if dump_file.exists() {
        if let Ok(metadata) = fs::metadata(&dump_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > CACHE_TTL_SECS {
                        let _ = fs::remove_file(&dump_file);
                        return None;
                    }
                }
            }
        }
        fs::read_to_string(dump_file).ok()
    } else {
        None
    }
}

fn prune_raw_dumps_if_needed(dir: &std::path::Path) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let mut files_with_mtime = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("log") {
            if let Ok(meta) = entry.metadata() {
                if let Ok(mtime) = meta.modified() {
                    if let Ok(elapsed) = mtime.elapsed() {
                        if elapsed.as_secs() > CACHE_TTL_SECS {
                            let _ = fs::remove_file(&path);
                            continue;
                        }
                    }
                    files_with_mtime.push((path, mtime));
                }
            }
        }
    }

    if files_with_mtime.len() > MAX_CACHE_ENTRIES {
        files_with_mtime.sort_by_key(|(_, mtime)| *mtime);
        let to_remove = files_with_mtime.len() - MAX_CACHE_ENTRIES;
        for (path, _) in files_with_mtime.into_iter().take(to_remove) {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_and_retrieve_raw_dump() {
        let payload = "Raw crash stack trace with critical user data\nError: line 42";
        let uri = save_raw_dump(payload);
        assert!(uri.starts_with("sha256:"));

        let retrieved = get_raw_dump(&uri);
        assert_eq!(retrieved.as_deref(), Some(payload));

        let retrieved_raw_hash = get_raw_dump(uri.trim_start_matches("sha256:"));
        assert_eq!(retrieved_raw_hash.as_deref(), Some(payload));
    }
}
