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
    // VULN-03: Use SHA-256 instead of DefaultHasher to prevent hash collision attacks
    let mut hasher = Sha256::new();
    hasher.update(payload.as_bytes());
    let result = hasher.finalize();
    let mut hash_str = String::new();
    for byte in result {
        hash_str.push_str(&format!("{:02x}", byte));
    }
    hash_str
}

pub fn get_cached_response(payload: &str) -> Option<String> {
    let dir = get_cache_dir()?;
    let hash = hash_payload(payload);
    let cache_file = dir.join(format!("{}.txt", hash));

    if cache_file.exists() {
        // TTL: Abaikan cache yang berusia lebih dari 24 jam
        if let Ok(metadata) = std::fs::metadata(&cache_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() > 86400 {
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

pub fn save_cached_response(payload: &str, response: &str) {
    if let Some(dir) = get_cache_dir() {
        let hash = hash_payload(payload);
        let cache_file = dir.join(format!("{}.txt", hash));
        let _ = fs::write(cache_file, response);
    }
}
